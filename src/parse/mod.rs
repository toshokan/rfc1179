mod raw;
use crate::lpd::*;

pub struct Parser;

#[derive(Debug)]
pub enum ParseError {
    Other
}

pub trait Parse {
    type Error;

    fn parse_lpr_command(i: &str) -> Result<LprCommand, Self::Error>;
    fn parse_receive_job_subcommand(i: &str) -> Result<ReceiveJobSubcommand, Self::Error>;
    fn parse_control_file_line(i: &str) -> Result<ControlFileLine, Self::Error>;
}

impl Parse for Parser {
    type Error = ParseError;

    fn parse_lpr_command(i: &str) -> Result<LprCommand, Self::Error> {
        raw::lpr_command_parser(i).map(|(_, x)| x).map_err(|e| {
	    eprintln!("pe = {:?}", e);
	    ParseError::Other
	})
    }

    fn parse_receive_job_subcommand(i: &str) -> Result<ReceiveJobSubcommand, Self::Error> {
        raw::receive_job_subcommand_parser(i)
            .map(|(_, x)| x)
            .map_err(|e| {
		eprintln!("pe = {:?}", e);
		ParseError::Other
	    })
    }

    fn parse_control_file_line(i: &str) -> Result<ControlFileLine, Self::Error> {
        raw::control_file_line_parser(i)
            .map(|(_, x)| x)
            .map_err(|e| {
		eprintln!("pe = {:?}", e);
		ParseError::Other
	    })
    }
}
