#[derive(Debug)]
pub struct QueueName(pub String);
#[derive(Debug)]
pub enum LprCommand {
    PrintWaiting(QueueName),
    ReceivePrinterJob(QueueName),
    SendQueueStateShort(QueueName, Vec<String>),
    SendQueueStateLong(QueueName, Vec<String>),
    RemoveJobs(QueueName, String, Vec<String>),
}

#[derive(Debug)]
pub enum ReceiveJobSubcommand {
    AbortJob,
    ReceiveControlFile(usize, String),
    ReceiveDataFile(usize, String),
}

#[derive(Debug)]
pub enum TroffFontKind {
    R,
    I,
    B,
    S,
}

#[derive(Debug)]
pub enum PrintKind {
    BannerPage,
    FormattedFile,
    FileLeavingControlCharacters,
    DitroffOutputFile,
    PostscriptOutputFile,
    FileWithPrFormat,
    FileWithFortranCarriageControl,
    TroffOutputFile,
    RasterFile,
}

#[derive(Debug)]
pub enum PlotKind {
    CifFile,
    DviFile,
    File,
}

#[derive(Debug)]
pub enum ControlFileLine {
    ClassForBannerPage(String),
    HostName(String),
    IndentPrinting(usize),
    JobNameForBannerPage(String),
    MailWhenPrinted(String),
    NameOfSourceFile(String),
    UserIdentification(String),
    SymbolicLinkData(usize, usize),
    TitleForPr(String),
    UnlinkDataFile(String),
    WidthOfOutput(usize),
    TroffFont(TroffFontKind, String),
    PrintFile(PrintKind, String),
    PlotFile(PlotKind, String),
    ReservedK,
    ReservedZ,
}

#[derive(Debug, Default)]
pub struct ControlFile {
    pub lines: Vec<ControlFileLine>
}

#[derive(Default)]
pub struct DataFile {
    pub data: Vec<u8>
}

impl std::fmt::Debug for DataFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	write!(f, "DataFile(...)")
    }
}
