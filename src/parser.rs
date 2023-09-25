use crate::record::{Level, LogcatRecord, ProcessRecord};
use chrono::{prelude::*, LocalResult};
use nom::bytes::complete::{tag, take, take_until1};
use nom::character::complete::{char, i32, multispace1, u32};

use nom::combinator::{opt, peek, rest};
use nom::error::{Error, ErrorKind};
use nom::sequence::{terminated, tuple};
use nom::{IResult, Parser};

pub struct LogcatParser {}
fn parse_year(s: &str) -> IResult<&str, i32> {
    let (_line, i2) = peek(take::<usize, &str, Error<_>>(4usize))(s)?;

    let (_line, value) = i32(i2)?;
    let (line, _) = take(4usize)(s)?;
    let (line, _) = char('-')(line)?;
    Ok((line, value))
}
fn parse_timestamp(s: &str) -> IResult<&str, DateTime<Local>> {
    // 08-30 18:10:53.566
    //or
    // 2017-08-30 18:10:53.566
    // let (s,time_str) = terminated(take_until1("  "),tag("  "))(s)?;
    let (s, year) = opt(parse_year)(s)?;
    let year = year.unwrap_or(Local::now().year());
    let (s, month) = terminated(u32, tag("-"))(s)?;
    let (s, day) = terminated(u32, tag(" "))(s)?;
    let (s, hour) = terminated(u32, tag(":"))(s)?;
    let (s, minute) = terminated(u32, tag(":"))(s)?;
    let (s, second) = terminated(u32, tag("."))(s)?;
    let (s, _millisecond) = terminated(u32, multispace1)(s)?;
    let time = Local.with_ymd_and_hms(year, month, day, hour, minute, second);
    match time {
        LocalResult::None => Err(nom::Err::Error(Error::new(s, ErrorKind::Eof))),
        LocalResult::Single(t) => {
            let time = t;
            Ok((s, time))
        }
        LocalResult::Ambiguous(t1, _t2) => {
            let time = t1;
            Ok((s, time))
        }
    }
}
impl LogcatParser {
    // use nom to parse logcat output
    //08-30 18:10:53.566  1904  6916 D NetworkMonitor/139: PROBE_DNS connect.rom.miui.com 27ms OK 111.13.141.125,39.156.150.112,39.156.150.3,111.13.141.31
    pub fn try_parse(&self, line: &str) -> Option<LogcatRecord> {
        let (_s, (timestamp, pid, tid, level, tag, message)) = tuple((
            parse_timestamp,
            terminated(u32, multispace1),
            terminated(u32, multispace1),
            terminated(take(1usize), multispace1).map(Level::from),
            terminated(take_until1(":"), tag(": ")),
            // take_until eof
            rest,
        ))(line)
        .ok()?;
        // println!("s:{:?},timestamp:{:?},pid:{:?},tid:{:?},level:{:?},tag:{:?},message:{:?}",s,timestamp,pid,tid,level,tag,message);
        Some(LogcatRecord {
            timestamp: Some(timestamp),
            pid,
            raw: line.to_string(),
            tag: tag.to_string(),
            message: message.to_string(),
            tid,
            level,
            ..LogcatRecord::default()
        })
    }
}

pub struct PSParser {}
impl PSParser {
    // use nom to parse ps output
    //user          pid  ppid  vsize  rss   wchan            pc  name
    // u0_a153      24103   772 16935184 232896 0                  0 S com.google.android.GoogleCamera
    pub fn try_parse(&self, line: &str) -> Option<ProcessRecord> {
        let (_s, (user, pid, ppid, _vsize, rss, _wchan, _addr, pc, name)) = tuple((
            terminated(take_until1(" "), multispace1::<&str, Error<_>>),
            terminated(u32, multispace1),
            terminated(u32, multispace1),
            terminated(u32, multispace1),
            terminated(u32, multispace1),
            terminated(take_until1(" "), multispace1),
            terminated(take_until1(" "), multispace1),
            terminated(take_until1(" "), multispace1),
            rest,
        ))(line)
        .ok()?;
        // println!("s:{:?},user:{:?},pid:{:?},ppid:{:?},vsize:{:?},rss:{:?},wchan:{:?},pc:{:?},name:{:?}",s,user,pid,ppid,vsize,rss,wchan,pc,name);
        Some(ProcessRecord {
            user: user.to_string(),
            pid,
            ppid,
            rss,
            pc: pc.to_string(),
            name: name.to_string(),
            ..ProcessRecord::default()
        })
    }
}
#[test]
fn parse_logcat_line() {
    let line = "08-30 18:10:53.566  1904  6916 D NetworkMonitor/139: PROBE_DNS connect.rom.miui.com 27ms OK";
    let res = LogcatParser {}.try_parse(line).unwrap();
    println!("{:?}", res);
    assert_eq!(res.timestamp.unwrap().day(), 30);
    assert_eq!(res.timestamp.unwrap().hour(), 18);
    assert_eq!(res.timestamp.unwrap().minute(), 10);
    assert_eq!(res.timestamp.unwrap().second(), 53);

    assert_eq!(res.pid, 1904);
    assert_eq!(res.tid, 6916);
    assert_eq!(res.level, Level::Debug);
    assert_eq!(res.tag, "NetworkMonitor/139");
    assert_eq!(res.message, "PROBE_DNS connect.rom.miui.com 27ms OK");
}
#[test]
fn parse_logcat_line_y() {
    let line = "2018-08-30 18:10:53.566  1904  6916 D NetworkMonitor/139: PROBE_DNS connect.rom.miui.com 27ms OK";
    let res = LogcatParser {}.try_parse(line).unwrap();
    println!("{:?}", res);
    assert_eq!(res.timestamp.unwrap().day(), 30);
    assert_eq!(res.timestamp.unwrap().hour(), 18);
    assert_eq!(res.timestamp.unwrap().minute(), 10);
    assert_eq!(res.timestamp.unwrap().second(), 53);

    assert_eq!(res.pid, 1904);
    assert_eq!(res.tid, 6916);
    assert_eq!(res.level, Level::Debug);
    assert_eq!(res.tag, "NetworkMonitor/139");
    assert_eq!(res.message, "PROBE_DNS connect.rom.miui.com 27ms OK");
}

#[test]
fn parse_ps_line() {
    let line = "u0_a153      24103   772 16935184 232896 0                  0 S com.google.android.GoogleCamera";
    let res = PSParser {}.try_parse(line).unwrap();
    println!("{:?}", res);
    assert_eq!(res.user, "u0_a153");
    assert_eq!(res.pid, 24103);
    assert_eq!(res.ppid, 772);
    assert_eq!(res.rss, 232896);
    assert_eq!(res.name, "com.google.android.GoogleCamera");
    assert_eq!(res.pc, "S");
}
