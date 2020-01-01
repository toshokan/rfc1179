use async_std::io::prelude::*;

pub mod lpd;
pub mod parse;

#[derive(Debug)]
pub enum LpdError {
    Io(std::io::Error),
    Parse(parse::ParseError),
    BadAck,
}

impl From<std::io::Error> for LpdError {
    fn from(error: std::io::Error) -> Self {
	LpdError::Io(error)
    }
}

impl From<parse::ParseError> for LpdError {
    fn from(error: parse::ParseError) -> Self {
	LpdError::Parse(error)
    }
}

#[derive(Debug, Default)]
pub struct Log {
    c: Vec<lpd::LprCommand>,
    jobs: Vec<Job>,
}

pub struct Worker<'s, R, W> {
    log: Log,
    inner: Inner<'s, R, W>
}

impl<'s, R, W> Worker<'s, R, W>
    where R: BufRead + std::marker::Unpin,
	  W: Write + std::marker::Unpin,
{
    pub fn new(r: &'s mut R, w: &'s mut W) -> Self {
	Self {
	    log: Log::default(),
	    inner: Inner {
		r,
		w
	    }
	}
    }

    pub async fn run<P: parse::Parse>(mut self) -> Result<Log, LpdError>
	where LpdError: From<P::Error>
    {
	use lpd::LprCommand::*;
	
	let mut buf = String::new();
	while self.inner.r.read_line(&mut buf).await? > 0 {
	    let command = self.inner.read_command::<P>(&buf)?;
	    if let ReceivePrinterJob(_) = &command {
		let job = self.inner.receive_job::<P>().await?;
		if let Some(job) = job {
		    self.log.jobs.push(job);
		}
	    }
	    self.log.c.push(command);
	    buf.clear();
	};
	
	Ok(self.log)
    }
}

struct Inner<'s, R, W> {
    r: &'s mut R,
    w: &'s mut W
}

#[derive(Debug, Default)]
struct Job {
    df: Option<lpd::DataFile>,
    cf: Option<lpd::ControlFile>
}

impl<'s, R, W> Inner<'s, R, W>
where R: BufRead + std::marker::Unpin,
      W: Write + std::marker::Unpin
{
    async fn positive_ack(&mut self) -> std::io::Result<()> {
	self.w.write(b"\x00").await?;
	Ok(())
    }

    async fn negative_ack(&mut self) -> std::io::Result<()> {
	self.w.write(b"\x01").await?;
	Ok(())
    }

    async fn assert_ack(&mut self) -> Result<(), LpdError> {
	let mut byte = [1; 1];
	self.r.read_exact(&mut byte).await?;
	match &byte[0] {
	    0_u8 => Ok(()),
	    _ => Err(LpdError::BadAck)
	}
    }

    fn read_command<P: parse::Parse>(&mut self, line: &str) -> Result<lpd::LprCommand, LpdError>
	where LpdError: From<P::Error>
    {
	Ok(P::parse_lpr_command(&line)?)
    }

    async fn receive_job<P: parse::Parse>(&mut self) -> Result<Option<Job>, LpdError>
	where LpdError: From<P::Error>
    {
	let mut buf = String::new();
	let mut job = Job::default();
	self.positive_ack().await?;
	while self.r.read_line(&mut buf).await? > 0 {
	    match P::parse_receive_job_subcommand(&buf)? {
		lpd::ReceiveJobSubcommand::AbortJob => return Ok(None),
		lpd::ReceiveJobSubcommand::ReceiveControlFile(b, _) => {
		    job.cf = Some(self.read_control_file::<P>(b).await?);
		},
		lpd::ReceiveJobSubcommand::ReceiveDataFile(b, _) => {
		    job.df = Some(self.read_data_file(b).await?);
		}
	    }
	    buf.clear();
	}
	Ok(Some(job))
    }

    async fn read_control_file<P: parse::Parse>(&mut self, bytes: usize) -> Result<lpd::ControlFile, LpdError>
	where LpdError: From<P::Error>
    {
	let mut file = lpd::ControlFile::default();
	let mut read_bytes = 0;
	while read_bytes < bytes {
	    let mut buf = String::new();
	    read_bytes += self.r.read_line(&mut buf).await?;
	    let line = P::parse_control_file_line(&buf)?;
	    file.lines.push(line)
	}
	self.assert_ack().await?;
	self.positive_ack().await?;
	Ok(file)
    }

    pub async fn read_data_file(&mut self, bytes: usize) -> Result<lpd::DataFile, LpdError> {
	let mut buf = vec![0; bytes];
	self.r.read_exact(&mut buf).await?;
	
	self.assert_ack().await?;
	self.positive_ack().await?;
	
	Ok(lpd::DataFile {
	    data: buf
	})
    }
}
