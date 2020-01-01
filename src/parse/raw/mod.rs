use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::sequence::tuple;

use nom::IResult;

use crate::lpd::*;
mod util;
use util::*;

fn print_waiting_parser(i: &str) -> IResult<&str, LprCommand> {
    simple_line_parser("\x01", &octet_seq_parser, |s| {
        LprCommand::PrintWaiting(QueueName(s.to_string()))
    })(i)
}

fn receive_printer_job_parser(i: &str) -> IResult<&str, LprCommand> {
    simple_line_parser("\x02", &octet_seq_parser, |s| {
        LprCommand::ReceivePrinterJob(QueueName(s.to_string()))
    })(i)
}

fn send_queue_state_parser<'a>(
    tag_text: &'a str,
    f: impl Fn(String, Vec<String>) -> LprCommand,
) -> impl Fn(&'a str) -> IResult<&str, LprCommand> {
    move |i: &'a str| {
        all_consuming(tuple((
            tag(tag_text),
            octet_seq_parser,
            whitespace_parser,
            list_parser,
            char('\n'),
        )))(i)
        .map(|(r, (_, s, _, l, _))| {
            let list = l.iter().map(ToString::to_string).collect();
            (r, f(s.to_string(), list))
        })
    }
}

fn send_queue_state_short_parser(i: &str) -> IResult<&str, LprCommand> {
    send_queue_state_parser("\x03", |s, l| {
        LprCommand::SendQueueStateShort(QueueName(s), l)
    })(i)
}

fn send_queue_state_long_parser(i: &str) -> IResult<&str, LprCommand> {
    send_queue_state_parser("\x04", |s, l| {
        LprCommand::SendQueueStateLong(QueueName(s), l)
    })(i)
}

fn remove_jobs_parser(i: &str) -> IResult<&str, LprCommand> {
    all_consuming(tuple((
        tag("\x05"),
        octet_seq_parser,
        whitespace_parser,
        octet_seq_parser,
        whitespace_parser,
        list_parser,
        char('\n'),
    )))(i)
    .map(|(r, (_, s, _, a, _, l, _))| {
        let list = l.iter().map(|s| s.to_string()).collect();
        (
            r,
            LprCommand::RemoveJobs(QueueName(s.to_string()), a.to_string(), list),
        )
    })
}

fn abort_job_parser(i: &str) -> IResult<&str, ReceiveJobSubcommand> {
    all_consuming(tuple((tag("\x01"), char('\n'))))(i)
        .map(|(r, _)| (r, ReceiveJobSubcommand::AbortJob))
}

fn receive_file_parser<'a>(
    tag_str: &'a str,
    f: impl Fn(usize, &'a str) -> ReceiveJobSubcommand,
) -> impl Fn(&'a str) -> IResult<&'a str, ReceiveJobSubcommand> {
    move |i: &'a str| {
        all_consuming(tuple((
            tag(tag_str),
            count_parser,
            whitespace_parser,
            octet_seq_parser,
            char('\n'),
        )))(i)
        .map(|(r, (_, c, _, n, _))| (r, f(c, n)))
    }
}

fn receive_control_file_parser(i: &str) -> IResult<&str, ReceiveJobSubcommand> {
    receive_file_parser("\x02", |c, n| {
        ReceiveJobSubcommand::ReceiveControlFile(c, n.to_string())
    })(i)
}

fn receive_data_file_parser(i: &str) -> IResult<&str, ReceiveJobSubcommand> {
    receive_file_parser("\x03", |c, n| {
        ReceiveJobSubcommand::ReceiveDataFile(c, n.to_string())
    })(i)
}

pub fn receive_job_subcommand_parser(i: &str) -> IResult<&str, ReceiveJobSubcommand> {
    nom::branch::alt((
        abort_job_parser,
        receive_control_file_parser,
        receive_data_file_parser,
    ))(i)
}

pub fn lpr_command_parser(i: &str) -> IResult<&str, LprCommand> {
    nom::branch::alt((
        print_waiting_parser,
        receive_printer_job_parser,
        send_queue_state_short_parser,
        send_queue_state_long_parser,
        remove_jobs_parser,
    ))(i)
}

fn class_for_banner_page_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("C", &octet_seq_parser, |s| {
        ControlFileLine::ClassForBannerPage(s.to_string())
    })(i)
}

fn host_name_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("H", &octet_seq_parser, |s| {
        ControlFileLine::HostName(s.to_string())
    })(i)
}

fn indent_printing_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("I", &count_parser, |c| ControlFileLine::IndentPrinting(c))(i)
}

fn job_name_for_banner_page_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("J", &octet_seq_parser, |s| {
        ControlFileLine::JobNameForBannerPage(s.to_string())
    })(i)
}

fn mail_when_printed_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("M", &octet_seq_parser, |s| {
        ControlFileLine::MailWhenPrinted(s.to_string())
    })(i)
}

fn name_of_source_file_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("N", &octet_seq_parser, |s| {
        ControlFileLine::NameOfSourceFile(s.to_string())
    })(i)
}

fn user_identification_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("P", &octet_seq_parser, |s| {
        ControlFileLine::UserIdentification(s.to_string())
    })(i)
}

fn symbolic_link_data_parser(i: &str) -> IResult<&str, ControlFileLine> {
    all_consuming(tuple((
        tag("S"),
        count_parser,
        whitespace_parser,
        count_parser,
        char('\n'),
    )))(i)
    .map(|(rem, (_, l, _, r, _))| (rem, ControlFileLine::SymbolicLinkData(l, r)))
}

fn title_for_pr_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("T", &octet_seq_parser, |s| {
        ControlFileLine::TitleForPr(s.to_string())
    })(i)
}

fn unlink_data_file_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("U", &octet_seq_parser, |s| {
        ControlFileLine::UnlinkDataFile(s.to_string())
    })(i)
}

fn width_of_output_parser(i: &str) -> IResult<&str, ControlFileLine> {
    simple_line_parser("W", &count_parser, |c| ControlFileLine::WidthOfOutput(c))(i)
}

fn troff_font_kind_parser(i: &str) -> IResult<&str, TroffFontKind> {
    nom::branch::alt((
        mini_parser(tag("1"), |_| TroffFontKind::R),
        mini_parser(tag("2"), |_| TroffFontKind::I),
        mini_parser(tag("3"), |_| TroffFontKind::B),
        mini_parser(tag("4"), |_| TroffFontKind::S),
    ))(i)
}

fn print_kind_parser(i: &str) -> IResult<&str, PrintKind> {
    nom::branch::alt((
        mini_parser(tag("L"), |_| PrintKind::BannerPage),
        mini_parser(tag("f"), |_| PrintKind::FormattedFile),
        mini_parser(tag("l"), |_| PrintKind::FileLeavingControlCharacters),
        mini_parser(tag("n"), |_| PrintKind::DitroffOutputFile),
        mini_parser(tag("o"), |_| PrintKind::PostscriptOutputFile),
        mini_parser(tag("p"), |_| PrintKind::FileWithPrFormat),
        mini_parser(tag("r"), |_| PrintKind::FileWithFortranCarriageControl),
        mini_parser(tag("t"), |_| PrintKind::TroffOutputFile),
        mini_parser(tag("v"), |_| PrintKind::RasterFile),
    ))(i)
}

fn plot_kind_parser(i: &str) -> IResult<&str, PlotKind> {
    nom::branch::alt((
        mini_parser(tag("c"), |_| PlotKind::CifFile),
        mini_parser(tag("d"), |_| PlotKind::DviFile),
        mini_parser(tag("g"), |_| PlotKind::CifFile),
    ))(i)
}

fn kinded_simple_line<'a, K, T>(
    kind_parser: &'a impl Fn(&'a str) -> IResult<&'a str, K>,
    f: impl Fn(K, &'a str) -> T,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    move |i: &'a str| {
        all_consuming(tuple((kind_parser, octet_seq_parser, char('\n'))))(i)
            .map(|(r, (k, s, _))| (r, f(k, s)))
    }
}

fn troff_font_parser(i: &str) -> IResult<&str, ControlFileLine> {
    kinded_simple_line(&troff_font_kind_parser, |k, s| {
        ControlFileLine::TroffFont(k, s.to_string())
    })(i)
}

fn print_file_parser(i: &str) -> IResult<&str, ControlFileLine> {
    kinded_simple_line(&print_kind_parser, |k, s| {
        ControlFileLine::PrintFile(k, s.to_string())
    })(i)
}

fn plot_file_parser(i: &str) -> IResult<&str, ControlFileLine> {
    kinded_simple_line(&plot_kind_parser, |k, s| {
        ControlFileLine::PlotFile(k, s.to_string())
    })(i)
}

fn reserved_k_parser(i: &str) -> IResult<&str, ControlFileLine> {
    tag("k")(i).map(|(r, _)| (r, ControlFileLine::ReservedK))
}

fn reserved_z_parser(i: &str) -> IResult<&str, ControlFileLine> {
    tag("z")(i).map(|(r, _)| (r, ControlFileLine::ReservedZ))
}

pub fn control_file_line_parser(i: &str) -> IResult<&str, ControlFileLine> {
    nom::branch::alt((
        class_for_banner_page_parser,
        host_name_parser,
        indent_printing_parser,
        job_name_for_banner_page_parser,
        mail_when_printed_parser,
        name_of_source_file_parser,
        user_identification_parser,
        symbolic_link_data_parser,
        title_for_pr_parser,
        unlink_data_file_parser,
        width_of_output_parser,
        troff_font_parser,
        print_file_parser,
        plot_file_parser,
        reserved_k_parser,
        reserved_z_parser,
    ))(i)
}
