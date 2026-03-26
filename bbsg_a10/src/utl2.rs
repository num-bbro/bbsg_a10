use crate::dcl::PeaSub;
//use crate::stg3::AojInfo;
use crate::utl3::ass_var_aoj;
use crate::utl3::ass_var_aoj_tr;
use bincode::{Decode, Encode};
use docx_rs::*;
use quick_xml::events::{/*BytesStart*/ Event};
use quick_xml::reader::Reader as XmlReader;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};
use zip::write::SimpleFileOptions;

#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct ArchiState {
    pub hdno: [usize; 5],
    pub hdfg: [bool; 5],
    pub figno: usize,
    pub tabno: usize,
}

pub static ARCHISTATE: LazyLock<Mutex<ArchiState>> = LazyLock::new(|| {
    Mutex::new(ArchiState {
        hdno: [0; 5],
        hdfg: [true, true, true, false, false],
        ..Default::default()
    })
});

pub fn modi_head(hds: &str, hi: usize) -> String {
    let mut arst = ARCHISTATE.lock().unwrap();
    let mut hdn = "".to_string();
    for ii in 0..5 {
        if ii == hi - 1 {
            arst.hdno[ii] += 1;
        }
        if ii > hi - 1 {
            arst.hdno[ii] = 0;
        }
        if ii < hi {
            if !hdn.is_empty() {
                hdn = format!("{hdn}.");
            }
            hdn = format!("{hdn}{}", arst.hdno[ii]);
        }
    }
    if arst.hdfg[hi - 1] {
        format!("{hdn}. {hds}")
    } else {
        hds.to_string()
    }
}

pub fn modi_fig(hds: &str) -> String {
    let mut arst = ARCHISTATE.lock().unwrap();
    arst.figno += 1;
    format!("ภาพที่ {}. {hds}", arst.figno)
}

pub fn modi_tab(hds: &str) -> String {
    let mut arst = ARCHISTATE.lock().unwrap();
    arst.tabno += 1;
    format!("ตารางที่ {}. {hds}", arst.tabno)
}

pub fn reg_test1() -> Result<(), Box<dyn std::error::Error>> {
    println!("1: {:?}", ar_cmd_split("||aaa||bbb||"));
    println!("2: {:?}", ar_cmd_split("ภาษาไทย"));
    println!("3: {:?}", ar_cmd_split("||h1||ภาษาไทย"));
    /*
    let cmd = regex::Regex::new(r"#([0-9a-zA-Z]+)#(.*)$")?;
    //let prm = regex::Regex::new(r"([0-9a-zA-Z]+):([0-9a-zA-Z]+):([0-9a-zA-Z]+):([0-9a-zA-Z]+)")?;
    //let prm = regex::Regex::new(r"([0-9a-zA-Z]+)")?;
    let prm = regex::Regex::new(r"\{(.+?)\}")?;
    //let caps = re.captures("#aaa#11:22:33:44:55").unwrap();
    let caps = cmd.captures("#aaa#{111}{222}{333}").unwrap();
    //let caps = cmd.captures("#aaa#").unwrap();
    let a1 = caps[1].to_string();
    let a2 = caps[2].to_string();
    println!("a1:{a1} a2:{a2}");
    for caps in prm.captures_iter(&a2) {
        let aa = caps.get(1).unwrap().as_str().to_string();
        println!("  = {aa}");
    }
    */
    Ok(())
}

pub fn get_cmd(line: &str) -> (String, Vec<String>) {
    let cmd = regex::Regex::new(r"#([0-9a-zA-Z]+)#(.*)$").unwrap();
    let prm = regex::Regex::new(r"\{(.+?)\}").unwrap();
    let Some(caps) = cmd.captures(line) else {
        return ("".to_string(), vec![]);
    };
    let cm = caps[1].to_string();
    let pm = caps[2].to_string();
    println!(" GET CMD: {cm} - {pm:?}");
    let mut pms = Vec::<String>::new();
    for caps in prm.captures_iter(&pm) {
        let aa = caps.get(1).unwrap().as_str().to_string();
        println!("  = {aa}");
        pms.push(aa);
    }
    (cm, pms)
}

pub fn create_docx_def0() -> Vec<DocxCompo> {
    let mut docv = Vec::<DocxCompo>::new();
    let style1 = Style::new("Heading1", StyleType::Paragraph).name("Heading 1");
    let style2 = Style::new("Heading2", StyleType::Paragraph).name("Heading 2");
    let style3 = Style::new("Heading3", StyleType::Paragraph).name("Heading 3");
    let header = Header::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("=== PEA Smart Grid Implementation Plan ===")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = Footer::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("---")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = footer.add_paragraph(
        Paragraph::new()
            .add_page_num(PageNum::new())
            .align(AlignmentType::Right),
    );

    docv.push(DocxCompo::Style(style1));
    docv.push(DocxCompo::Style(style2));
    docv.push(DocxCompo::Style(style3));
    docv.push(DocxCompo::Header(header));
    docv.push(DocxCompo::Footer(footer));
    docv
}

pub fn create_docx_def() -> Vec<DocxCompo> {
    let mut docv = Vec::<DocxCompo>::new();
    let style1 = Style::new("Heading1", StyleType::Paragraph).name("Heading 1");
    let style2 = Style::new("Heading2", StyleType::Paragraph).name("Heading 2");
    let style3 = Style::new("Heading3", StyleType::Paragraph).name("Heading 3");
    let header = Header::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("=== PEA Smart Grid Implementation Plan ===")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = Footer::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("---")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = footer.add_paragraph(
        Paragraph::new()
            .add_page_num(PageNum::new())
            .align(AlignmentType::Right),
    );
    let runp = RunProperty::new()
        .fonts(RunFonts::new().cs(STD_FONT))
        .size(32);
    let papr = ParagraphProperty::new().run_property(runp);
    let papr = papr.page_break_before(true);
    let toc = TableOfContents::new();
    let toc = toc.heading_styles_range(1, 2);
    //let toc = toc.heading_styles_range(1, 3);
    let toc = toc.paragraph_property(papr);
    let toc = toc.auto();
    //let cvimg = format!("{VW_IMG_PATH}cover1.png");
    let cvimg = format!("{VW_IMG_PATH}cover2.png");
    let mut img = std::fs::File::open(&cvimg).unwrap();
    let mut buf = Vec::new();
    let _ = img.read_to_end(&mut buf).unwrap();
    let pic = Pic::new(&buf)
        .size(700 * 9525, 1000 * 9525)
        .floating()
        .offset_x(0)
        .offset_y(0);

    let mut vtt = vec![];
    for _i in 0..13 {
        let tt = Paragraph::new().add_run(
            Run::new()
                .add_text(" ")
                .fonts(RunFonts::new().cs(STD_FONT))
                .size(60),
        );
        vtt.push(tt);
    }

    let tt = Paragraph::new().add_run(
        Run::new()
            .add_text("รายงานผลการวิเคราะห์ทางการเงินและเศรษฐศาสตร์")
            .fonts(RunFonts::new().cs(STD_FONT))
            .size(60),
    );
    vtt.push(tt);
    let tt = Paragraph::new().add_run(
        Run::new()
            .add_text("Financial and Economic Analysis Report")
            .fonts(RunFonts::new().cs(STD_FONT))
            .size(60),
    );
    vtt.push(tt);
    let tt = Paragraph::new().add_run(
        Run::new()
            .add_text("Smart Grid Implementation Plan")
            .fonts(RunFonts::new().cs(STD_FONT))
            .size(60),
    );
    vtt.push(tt);

    let pimg = Paragraph::new().add_run(Run::new().add_image(pic));

    let ptoc = Paragraph::new()
        .add_run(
            Run::new()
                .add_text("สารบัญ")
                .fonts(RunFonts::new().cs(STD_FONT))
                .size(40),
        )
        .page_break_before(true)
        .align(AlignmentType::Center);

    docv.push(DocxCompo::Style(style1));
    docv.push(DocxCompo::Style(style2));
    docv.push(DocxCompo::Style(style3));
    //docv.push(DocxCompo::Paragraph(pitt));
    for tt in vtt {
        docv.push(DocxCompo::Paragraph(tt));
    }
    docv.push(DocxCompo::Paragraph(pimg));
    docv.push(DocxCompo::Paragraph(ptoc));
    docv.push(DocxCompo::ToC(toc));
    docv.push(DocxCompo::Header(header));
    docv.push(DocxCompo::Footer(footer));
    docv
}

pub const INDIR: &str =
    "/mnt/c/Users/choom/Documents/wk33/peasg/archi3/PEA-SG-admin-2025-12-25.archimate";


pub fn archi5(vid: &str, fnm: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("vid: {vid}");
    archi_extract(INDIR, "archi")?;
    let (ar_elem, _ar_fold) = archi_ana1()?;
    //println!("elm:{} fold:{}", ar_elem.child.len(), _ar_fold.child.len());
    let mut ar_elmh = HashMap::<String, Rc<RefCell<ArchElem>>>::new();
    let mut ar_prof = HashMap::<String, String>::new();
    for e in ar_elem.child.iter() {
        let ee = e.borrow();
        let Some(id) = ee.attr.get("id") else {
            continue;
        };
        if ee.elem == "profile" {
            let Some(nm) = ee.attr.get("name") else {
                continue;
            };
            //println!("profile = '{nm}'");
            ar_prof.insert(nm.to_string(), id.to_string());
        }
        ar_elmh.insert(id.to_string(), e.clone());
    }
    let mut docv = create_docx_def0();
    docv = ar_gen_doc(docv, vid, &ar_elmh)?;
    //println!("ARCHI5 : {} ======================", docv.len());
    let fnm1 = format!("{fnm}-0.docx");
    let fnm2 = format!("{fnm}-1.docx");
    write_docx(docv, fnm1.as_str(), fnm2.as_str(), "temp")?;
    Ok(())
}

pub fn archi4(vid: &str, fnm: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("vid: {vid}");
    archi_extract(INDIR, "archi")?;
    let (ar_elem, _ar_fold) = archi_ana1()?;
    println!("elm:{} fold:{}", ar_elem.child.len(), _ar_fold.child.len());

    let mut ar_elmh = HashMap::<String, Rc<RefCell<ArchElem>>>::new();
    for e in ar_elem.child.iter() {
        let ee = e.borrow();
        let Some(id) = ee.attr.get("id") else {
            continue;
        };
        ar_elmh.insert(id.to_string(), e.clone());
    }
    let mut docv = create_docx_def();
    docv = ar_gen_doc(docv, vid, &ar_elmh)?;
    let fnm1 = format!("{fnm}-0.docx");
    let fnm2 = format!("{fnm}-1.docx");
    write_docx(docv, fnm1.as_str(), fnm2.as_str(), "temp")?;
    Ok(())
}

pub fn archi_vids(vid: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("vid: {vid}");
    //let indir = "/mnt/c/Users/choom/Documents/wk33/peasg/archi3/PEA-SG-admin-2025-10-15.archimate";
    archi_extract(INDIR, "archi")?;
    let (ar_elem, _ar_fold) = archi_ana1()?;
    println!("elm:{} fold:{}", ar_elem.child.len(), _ar_fold.child.len());

    let mut ar_elmh = HashMap::<String, Rc<RefCell<ArchElem>>>::new();
    for e in ar_elem.child.iter() {
        let ee = e.borrow();
        let Some(id) = ee.attr.get("id") else {
            continue;
        };
        ar_elmh.insert(id.to_string(), e.clone());
    }
    let vids = ar_view_ids(vec![], vid, &ar_elmh)?;
    for vi in &vids {
        println!("\"{vi}\",");
    }
    Ok(())
}

pub fn archi2(vid: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("vid: {vid}");
    //let indir = "/mnt/c/Users/choom/Documents/wk33/peasg/archi3/PEA-SG-admin-2025-10-15.archimate";
    archi_extract(INDIR, "archi")?;
    let (ar_elem, _ar_fold) = archi_ana1()?;
    println!("elm:{} fold:{}", ar_elem.child.len(), _ar_fold.child.len());

    let mut ar_elmh = HashMap::<String, Rc<RefCell<ArchElem>>>::new();
    for e in ar_elem.child.iter() {
        let ee = e.borrow();
        let Some(id) = ee.attr.get("id") else {
            continue;
        };
        ar_elmh.insert(id.to_string(), e.clone());
    }
    let vids = ar_view_ids(vec![], vid, &ar_elmh)?;
    println!("vids: len:{}", vids.len());
    let mut docv = create_docx_def();
    docv = ar_gen_doc(docv, vid, &ar_elmh)?;
    //println!("docv:{docv:?}");
    write_docx(docv, "gen1.docx", "gen2.docx", "temp")?;
    Ok(())
}

pub const STD_FONT: &str = "TH Sarabun New";

#[derive(Debug, Clone)]
pub enum DiagramType {
    None,
    ElemRef,
    ViewRef,
    NoteRef,
}

use docx_rs::Docx;

#[derive(Debug, Clone)]
pub struct Bound {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

#[derive(Debug, Clone)]
pub struct Diagram {
    pub eid: String,
    pub dgtp: DiagramType,
    pub bnd: Bound,
    pub rc: Rc<RefCell<ArchElem>>,
    pub xtp: String,
    pub prf: String,
    pub vtp: String,
}

pub fn check(bnd: &Bound, vw: &ArViewDocInfo) {
    println!("{bnd:?} {vw:?}");
    println!("{}", bnd.y1);
    println!("{}", vw.vnm);
}

pub fn txt_lines(doc: String) -> Vec<String> {
    let parts = doc.split("\n");
    let mut line = String::new();
    let mut lines = Vec::<String>::new();
    for ln in parts {
        let ll = ln.to_string();
        if ll.is_empty() {
            if !line.is_empty() {
                lines.push(line);
            }
            line = String::new();
        }
        line = format!("{line}{ll}");
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub fn ar_cmd_split(line: &str) -> Vec<String> {
    let mut pms = Vec::<String>::new();
    let pre = regex::Regex::new(r"\|([^\|]+)").unwrap();
    for caps in pre.captures_iter(line) {
        if let Some(cap) = caps.get(1) {
            let a = cap.as_str();
            pms.push(a.to_string());
        }
    }
    if pms.is_empty() {
        pms.push(line.to_string());
    }
    pms
}
//======================================================
pub fn footnote(
    mut docv: Vec<DocxCompo>,
    _vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let c1 = Paragraph::new().add_run(
        Run::new()
            .add_footnote_reference(
                Footnote::new()
                    .add_content(Paragraph::new().add_run(Run::new().add_text("EA: id name"))),
            )
            .size(20)
            .fonts(RunFonts::new().cs(STD_FONT)),
    );
    //.indent(Some(840), None, None, None);
    docv.push(DocxCompo::Paragraph(c1));
    Ok(docv)
}

//======================================================
pub fn doc0(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("doc0 {lst}");
    let c1 = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    //docv = add_docv(docv, c1, vw, ft)?;
    docv = add_docxv(docv, DocxCompo::Paragraph(c1), vw)?;
    Ok(docv)
}

//======================================================
pub fn dn(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    n: i32,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    let idn = n * 700;
    let n = n as usize;
    println!("doc1 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .outline_lvl(n)
        .indent(Some(idn), None, None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn hn(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    n: i32,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    if n == 2 {
        docv = sp(docv, vw)?;
    }
    let lst = pws.last().unwrap();
    let cnt = modi_head(lst.as_str(), n as usize);
    let hd = format!("Heading{n}");
    let fnsz = [48, 44, 40, 36, 32];
    let nn = if n > 5 { 5 } else { n };
    let nn = if nn < 1 { 5 } else { nn };
    let nn = fnsz[nn as usize - 1];
    let mut h1 = Paragraph::new().add_run(
        Run::new()
            .add_text(cnt)
            .size(nn)
            .fonts(RunFonts::new().cs(STD_FONT)),
    );
    h1 = h1.style(&hd);
    if n == 1 {
        h1 = h1.page_break_before(true);
    }
    docv = add_docxv(docv, DocxCompo::Paragraph(h1), vw)?;
    println!("hn {n} - {lst}");
    Ok(docv)
}

//======================================================
pub fn c1(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("c1 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    //docv = add_docv(docv, pa, vw, ft)?;
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn c2(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("c2 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    //docv = add_docv(docv, pa, vw, ft)?;
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn c3(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("c3 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn pg(mut docv: Vec<DocxCompo>, vw: &ArViewDocInfo) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    //let lst = pws.last().unwrap();
    //println!("sp");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(" ")
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .page_break_before(true)
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn sp(mut docv: Vec<DocxCompo>, vw: &ArViewDocInfo) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    //let lst = pws.last().unwrap();
    //println!("sp");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(" ")
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

use image::ImageReader;

pub const VW_IMG_PATH: &str = "/mnt/c/Users/choom/Documents/wk33/peasg/archi3/images/";
pub const DOCX_IMG_FAC: i32 = 9525;

//======================================================
pub fn img1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("img1 {}", vw.vid);
    let fimg = format!("{VW_IMG_PATH}{}.png", vw.vid);
    if std::path::Path::new(&fimg).exists() {
        println!("file {fimg} exists");
        if let Ok(img) = ImageReader::open(&fimg) {
            //println!("image opened");
            if let Ok(img) = img.decode() {
                //println!("image decoded");
                let (mut w, mut h) = (img.width(), img.height());
                //println!("fimg: {fimg} {w},{h}");
                let mut img = std::fs::File::open(fimg).unwrap();
                let mut buf = Vec::new();
                let _ = img.read_to_end(&mut buf).unwrap();
                //let pic = Pic::new(&buf).size(320 * 9525, 240 * 9525);
                let w0 = 640;
                if w > w0 {
                    h = h * w0 / w;
                    w = w0;
                }
                let pic = Pic::new(&buf).size(w * 9525, h * 9525);
                //let ppic = Paragraph::new().add_run(Run::new().add_image(pic.clone()));
                let ppic = Paragraph::new()
                    .add_run(Run::new().add_image(pic.clone()))
                    .align(AlignmentType::Center);
                docv = add_docxv(docv, DocxCompo::Paragraph(ppic), vw)?;
            }
        }
        let lst = modi_fig(lst.as_str());
        let pa = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(lst)
                    .size(32)
                    .fonts(RunFonts::new().cs(STD_FONT)),
            )
            .align(AlignmentType::Center);
        docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    } else {
        println!("IMG {fimg} not found");
    }
    Ok(docv)
}

use crate::img::fda01::get_img;

//======================================================
pub fn img2(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("IMG2 {pws:?}");
    let sc = script_param(pws);
    let lst = pws.last().unwrap();
    let mut fdid = "KLO03".to_string();
    if sc.fdid.len() == 5 {
        fdid = sc.fdid.to_string();
    }
    let dnm = crate::dcl::get_dirnm();
    let f02 = format!("{dnm}/fdimg-road/{fdid}-rd02.jpeg");
    let buf = get_img(fdid.as_str(), "roadmap", f02.as_str()).unwrap_or_default();
    //println!("buf: {}", buf.len());
    if let Ok(img) = ImageReader::open(&f02) {
        //println!("image opened");
        if let Ok(img) = img.decode() {
            //println!("image decoded");
            let (mut w, mut h) = (img.width(), img.height());
            //println!("  {w},{h}");
            let w0 = 640;
            if w > w0 {
                h = h * w0 / w;
                w = w0;
            }
            let pic = Pic::new(&buf).size(w * 9525, h * 9525);
            let ppic = Paragraph::new()
                .add_run(Run::new().add_image(pic.clone()))
                .align(AlignmentType::Center);
            docv = add_docxv(docv, DocxCompo::Paragraph(ppic), vw)?;
        }
    }
    let lst = modi_fig(lst.as_str());
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .align(AlignmentType::Center);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn img3(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("IMG3 {pws:?}");
    let sc = script_param(pws);
    let lst = pws.last().unwrap();
    let mut fdid = "KLO03".to_string();
    if sc.fdid.len() == 5 {
        fdid = sc.fdid.to_string();
    }
    let dnm = crate::dcl::get_dirnm();
    let f02 = format!("{dnm}/fdimg-sate/{fdid}-rd02.jpeg");
    let buf = get_img(fdid.as_str(), "satellite", f02.as_str()).unwrap_or_default();
    //println!("buf: {}", buf.len());
    if let Ok(img) = ImageReader::open(&f02) {
        //println!("image opened");
        if let Ok(img) = img.decode() {
            //println!("image decoded");
            let (mut w, mut h) = (img.width(), img.height());
            //println!("  {w},{h}");
            let w0 = 640;
            if w > w0 {
                h = h * w0 / w;
                w = w0;
            }
            let pic = Pic::new(&buf).size(w * 9525, h * 9525);
            let ppic = Paragraph::new()
                .add_run(Run::new().add_image(pic.clone()))
                .align(AlignmentType::Center);
            docv = add_docxv(docv, DocxCompo::Paragraph(ppic), vw)?;
        }
    }
    let lst = modi_fig(lst.as_str());
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .align(AlignmentType::Center);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

#[derive(Debug, Clone)]
pub struct ArViewDocInfo<'a> {
    vid: &'a str,
    vnm: String,
    nms: String,
    es: &'a HashMap<String, Rc<RefCell<ArchElem>>>,
    vdia: &'a Vec<Vec<Diagram>>,
    edia: &'a Vec<Vec<Diagram>>,
    adia: &'a Vec<Vec<Diagram>>,
}

pub fn end_ref(docv: Vec<DocxCompo>, vw: &ArViewDocInfo) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut ii = 0;
    for (i, doc) in docv.iter().enumerate() {
        if let DocxCompo::Paragraph(_) = doc {
            ii = i;
        }
    }
    let mut vdoc = Vec::<DocxCompo>::new();
    for (i0, doc) in docv.into_iter().enumerate() {
        let doc = if i0 == ii {
            if let DocxCompo::Paragraph(mut par) = doc {
                let v = &vw.nms[6..];
                //let rf = format!("ea: '{v}' {}", vw.vid);
                let rf = format!("ea:{v}");
                par = par.add_run(
                    Run::new()
                        .add_footnote_reference(
                            Footnote::new().add_content(
                                Paragraph::new().add_run(
                                    Run::new()
                                        .add_text(rf)
                                        .size(16)
                                        .fonts(RunFonts::new().cs(STD_FONT)),
                                ),
                            ),
                        )
                        .size(12)
                        .fonts(RunFonts::new().cs(STD_FONT)),
                );
                DocxCompo::Paragraph(par)
            } else {
                doc
            }
        } else {
            doc
        };
        vdoc.push(doc);
    }
    Ok(vdoc)
}

//=======================================================
//
pub fn add_docxv(
    mut docv: Vec<DocxCompo>,
    docx: DocxCompo,
    _vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    docv.push(docx);
    Ok(docv)
}

pub fn data0(_sc: &ScriptParam) -> Vec<Vec<String>> {
    vec![
        vec![
            "1a".to_string(),
            "1b".to_string(),
            "1c".to_string(),
            "1d".to_string(),
        ],
        vec![
            "2a".to_string(),
            "2b".to_string(),
            "2c".to_string(),
            "2d".to_string(),
        ],
    ]
}

pub const DOC_TB_INDENT: i32 = 500;
pub const TABSHADE: &str = "#ecffe6";
pub const CELL_MARGIN: i32 = 100;

//======================================================
pub fn v1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("view 1");
    let _sc = script_param(pws);
    for dis in vw.vdia.iter() {
        for di in dis.iter() {
            docv = ar_gen_doc(docv, &di.eid, vw.es)?;
        }
    }
    Ok(docv)
}

//======================================================
pub fn v2(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &mut ArViewDocInfo,
    hd: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("view 1");
    let sc = script_param(pws);
    for dis in vw.adia.iter() {
        for di in dis.iter() {
            match di.dgtp {
                DiagramType::ViewRef => {
                    println!("SHOW VIEW >>>>>>>>>>>>>>>>>>>>>>>>");
                    docv = ar_gen_doc(docv, &di.eid, vw.es)?;
                }
                DiagramType::ElemRef => {
                    let mut xtp = if di.xtp.starts_with("archimate:") {
                        di.xtp[10..].to_string()
                    } else {
                        "".to_string()
                    };
                    if !di.prf.is_empty() {
                        xtp = di.prf.to_string();
                    }
                    if sc.xtype.is_empty() || sc.xtype.contains(&xtp) {
                        docv = elm_to_render(docv, di.rc.clone(), vw, hd)?;
                    }
                }
                DiagramType::NoteRef => {}
                DiagramType::None => {}
            }
        }
    }
    Ok(docv)
}

//======================================================
pub fn e2(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &mut ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let _sc = script_param(pws);
    for dis in vw.edia.iter() {
        for di in dis.iter() {
            docv = elm_to_render(docv, di.rc.clone(), vw, true)?;
        }
    }
    Ok(docv)
}

//======================================================
pub fn e1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let _sc = script_param(pws);
    for dis in vw.edia.iter() {
        for di in dis.iter() {
            if let Some(e) = vw.es.get(&di.eid) {
                let e = e.borrow();
                let vnm = if let Some(n) = e.attr.get("name") {
                    n.to_string()
                } else {
                    "?".to_string()
                };
                let doc = get_elem_doc(&e);
                let doc = format!("{vnm}: {doc}");
                let pws = vec![doc];
                docv = dn(docv, &pws, 2, vw)?;
            }
        }
    }
    Ok(docv)
}

//
//======================================================
pub fn elm_to_render(
    mut docv: Vec<DocxCompo>,
    rc: Rc<RefCell<ArchElem>>,
    vw: &mut ArViewDocInfo,
    hd: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let r = rc.borrow();
    let mut eid = "?".to_string();
    eid = r.attr.get("archimateElement").unwrap_or(&eid).to_string();
    //println!("eid:{eid} chd:{}", r.child.len());
    if let Some(e) = vw.es.get(&eid) {
        let e = e.borrow();
        let enm = "".to_string();
        let enm = e.attr.get("name").unwrap_or(&enm);
        let enm = enm.trim();
        let doc = get_elem_doc(&e);
        let doc = doc.replace("$name", enm);
        //let ses = get_elem_prop(&e, "rdfs:seeAlso");
        if !enm.is_empty() && !doc.is_empty() {
            docv = sp(docv, vw)?;
        }
        if !enm.is_empty() && hd {
            let pa = Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(enm)
                            .size(32)
                            .bold()
                            .fonts(RunFonts::new().cs(STD_FONT)),
                    )
                    //.indent(Some(idn), None, None, None)
                    ;
            docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
        }
        if !doc.is_empty() {
            let lines = txt_lines(doc);
            docv = ar_line_doc(docv, &lines, vw, false)?;
        }
    }
    for r in &r.child {
        docv = elm_to_render(docv, r.clone(), vw, hd)?;
    }
    Ok(docv)
}

//======================================================
#[derive(Debug, Clone, Default)]
pub struct ScriptParam {
    pub hds: Vec<String>,
    pub mgs: Vec<i32>,
    pub wds: Vec<usize>,
    pub yrs: Vec<usize>,
    pub als: Vec<AlignmentType>,
    pub fld: Vec<String>,
    pub pan: Vec<String>,
    pub lmt: usize,
    pub cmd: Option<String>,
    pub sum: bool,
    pub pvid: String,
    pub sbid: String,
    pub fdid: String,
    pub xtype: Vec<String>,
    pub aojcd: Vec<String>,
    pub max2min: Vec<String>,
    pub min2max: Vec<String>,
}

pub fn script_param(pws: &[String]) -> ScriptParam {
    let prex = regex::Regex::new(r"([0-9a-zA-Z]+)=(.*)$").unwrap();
    let rx_hd = regex::Regex::new(r"([^,]+)").unwrap();
    let rx_wd = regex::Regex::new(r"([0-9][^,]+)").unwrap();
    let rx_al = regex::Regex::new(r"(Left|Center|Right[^,]*)").unwrap();
    let mut sc = ScriptParam::default();
    for w in pws.iter().skip(1).take(pws.len() - 1) {
        let Some(caps) = prex.captures(w) else {
            continue;
        };
        let ky = caps[1].to_string();
        let va = caps[2].to_string();
        //println!("{i}. {w} => {ky} = {va}");
        match ky.as_str() {
            "cmd" => sc.cmd = Some(va),
            "sum" => sc.sum = va == "sum",
            "head" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    let parts = aa.split(":");
                    let mut p_iter = parts.into_iter();
                    let hd = p_iter.next().unwrap();
                    let cn = if let Some(w) = p_iter.next() {
                        w.parse::<i32>().unwrap()
                    } else {
                        1
                    };
                    //println!("    hd:{hd} cn:{cn}");
                    sc.hds.push(hd.to_string());
                    sc.mgs.push(cn);
                }
            }
            "year" => {
                let nos = va.split(",");
                for n in nos {
                    //println!("  {n}");
                    sc.yrs.push(n.parse::<usize>().unwrap());
                }
            }
            "width" => {
                for caps in rx_wd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    sc.wds.push(aa.parse::<usize>().unwrap());
                }
            }
            "align" => {
                for caps in rx_al.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    let al = match aa.as_str() {
                        "Left" => AlignmentType::Left,
                        "Center" => AlignmentType::Center,
                        "Right" => AlignmentType::Right,
                        _ => AlignmentType::Left,
                    };
                    sc.als.push(al);
                }
            }
            "field" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    let parts = aa.split(":");
                    let mut p_iter = parts.into_iter();
                    let fd = p_iter.next().unwrap();
                    let pn = if let Some(p) = p_iter.next() {
                        p.to_string()
                    } else {
                        "0".to_string()
                    };
                    //println!("    hd:{fd} cn:{pn}");
                    sc.fld.push(fd.to_string());
                    sc.pan.push(pn.to_string());
                }
            }
            "limit" => {
                sc.lmt = va.parse::<usize>().unwrap_or_default();
            }
            "pvid" => {
                sc.pvid = va.to_string();
            }
            "sbid" => {
                sc.sbid = va.to_string();
            }
            "fdid" => {
                sc.fdid = va.to_string();
            }
            "xtype" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.xtype.push(aa);
                }
            }
            "aojcd" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.aojcd.push(aa);
                }
            }
            "max2min" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.max2min.push(aa);
                }
                println!("<<<<<<<<<<<<<<<<<<< MAX2MIN {:?}", sc.max2min);
            }
            "min2max" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.min2max.push(aa);
                }
                println!("<<<<<<<<<<<<<<<<<<< MIN2MAX {:?}", sc.min2max);
            }
            a => {
                println!("======= UNKNOWN para {a} = {va}");
            }
        }
    }
    sc
}

use crate::dcl::Pan;
use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use strum::IntoEnumIterator;

pub fn fld_2_var(nm: &str) -> VarType {
    for v in VarType::iter() {
        let vs = format!("{v:?}");
        if nm == vs {
            return v;
        }
    }
    VarType::None
}

pub fn val_2_form(v: f32, pn: &str) -> String {
    match pn {
        "1" => v.pan(1),
        "2" => v.pan(2),
        "3" => v.pan(3),
        "P" => {
            let v = v * 100.0;
            format!("{}%", v.pan(1))
        }
        "K" => {
            let v = v / 1_000.0;
            v.pan(1)
        }
        "M" => {
            let v = v / 1_000_000.0;
            v.pan(1)
        }
        _ => v.pan(0),
    }
}

pub fn elm_to_rows(
    mut rows: Vec<Vec<String>>,
    rc: Rc<RefCell<ArchElem>>,
    vw: &mut ArViewDocInfo,
) -> Vec<Vec<String>> {
    let r = rc.borrow();
    let mut eid = "?".to_string();
    eid = r.attr.get("archimateElement").unwrap_or(&eid).to_string();
    if let Some(e) = vw.es.get(&eid) {
        let e = e.borrow();
        let enm = "".to_string();
        let enm = e.attr.get("name").unwrap_or(&enm).to_string();
        let doc = get_elem_doc(&e);
        let no = format!("{}", rows.len());
        if !enm.is_empty() && !doc.is_empty() {
            rows.push(vec![no, enm, doc]);
        }
    }
    for r in &r.child {
        rows = elm_to_rows(rows, r.clone(), vw);
    }
    rows
}

pub fn diag_to_rows(sc: &ScriptParam, vw: &mut ArViewDocInfo) -> Vec<Vec<String>> {
    //println!("DIAG DOC : {sc:?}");
    let mut rows = Vec::<Vec<String>>::new();
    //for dis in vw.edia.iter() {
    for dis in vw.adia.iter() {
        for di in dis.iter() {
            match di.dgtp {
                DiagramType::ViewRef => {
                    //docv = ar_gen_doc(docv, &di.eid, vw.es)?;
                }
                DiagramType::ElemRef => {
                    let mut xtp = if di.xtp.starts_with("archimate:") {
                        di.xtp[10..].to_string()
                    } else {
                        "".to_string()
                    };
                    if !di.prf.is_empty() {
                        xtp = di.prf.to_string();
                    }
                    if sc.xtype.is_empty() || sc.xtype.contains(&xtp) {
                        rows = elm_to_rows(rows, di.rc.clone(), vw);
                    }
                }
                DiagramType::NoteRef => {}
                DiagramType::None => {}
            }
        }
    }
    rows
}

pub fn ass_reorder(assv0: &mut [PeaAssVar], sc: &ScriptParam) {
    let mut done = false;
    if !sc.max2min.is_empty() {
        let ei = fld_2_var(sc.max2min[0].as_str());
        if let VarType::None = ei {
        } else {
            println!(">>>>>>>>>>>>>>>>>>>>> MAX2MIN ORDER ");
            assv0.sort_by(|b, a| {
                let a0 = a.v[ei.tousz()].v;
                let b0 = b.v[ei.tousz()].v;
                a0.partial_cmp(&b0).unwrap()
            });
            done = true;
        }
    } else if !sc.min2max.is_empty() {
        let ei = fld_2_var(sc.min2max[0].as_str());
        if let VarType::None = ei {
        } else {
            println!(">>>>>>>>>>>>>>>>>>>>> MIN2MAX ORDER ");
            assv0.sort_by(|a, b| {
                let a0 = a.v[ei.tousz()].v;
                let b0 = b.v[ei.tousz()].v;
                a0.partial_cmp(&b0).unwrap()
            });
            done = true;
        }
    }
    if !done {
        assv0.sort_by(|a, b| {
            let a0 = a.v[VarType::Uc1Rank.tousz()].v
                + a.v[VarType::Uc2Rank.tousz()].v
                + a.v[VarType::Uc3Rank.tousz()].v;
            let b0 = b.v[VarType::Uc1Rank.tousz()].v
                + b.v[VarType::Uc2Rank.tousz()].v
                + b.v[VarType::Uc3Rank.tousz()].v;
            a0.partial_cmp(&b0).unwrap()
        });
    }
}

use crate::dcl::BranchGIS;

pub fn tab_row_popu(sc: &mut ScriptParam, assv0: &[PeaAssVar], nmtp: &str) -> Vec<Vec<String>> {
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["".to_string()]];
    let buf = std::fs::read(format!("{dnm}/000-subm.bin")).unwrap();

    //======== SUB INFO
    let Ok((subm, _)): Result<(HashMap<String, PeaSub>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };

    //======== AOJ INFO
    let buf = std::fs::read(format!("{dnm}/000-aojm.bin")).unwrap();
    //let Ok((aojm, _)): Result<(HashMap<String, AojInfo>, usize), _> =
    let Ok((aojm, _)): Result<(HashMap<String, BranchGIS>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };

    if !sc.yrs.is_empty() {
        let mut hds = Vec::<String>::new();
        for i in 0..(sc.fld.len() - 1) {
            let hd = if i < sc.hds.len() {
                sc.hds[i].to_string()
            } else {
                sc.fld[i].to_string()
            };
            hds.push(hd);
        }
        for y in &sc.yrs {
            hds.push(format!("ปี{}", y + 2026));
        }
        sc.hds = hds;
    }
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        let nm = match nmtp {
            "P" => a.pvid.to_string(),
            "S" => {
                if let Some(psb) = subm.get(&a.sbid) {
                    format!("{} - {}", a.sbid, psb.name)
                } else {
                    a.sbid.to_string()
                }
            }
            "A" => {
                if let Some(ao) = aojm.get(&a.aoj) {
                    let anm = ao.name.clone().unwrap_or(String::new()).to_string();
                    format!("{} - {anm}", a.aoj)
                } else {
                    a.aoj.to_string()
                }
            }
            "T" => a.peano.to_string(),
            _ => "?".to_string(),
        };
        v.push(nm);
        //v.push(a.pvid.clone());
        let mut vas = Vec::<f32>::new();
        let mut tk = sc.fld.len();
        if !sc.yrs.is_empty() {
            tk -= 1;
        }
        //println!("======= TB FIELD : {:?}", sc.fld);
        for (i, (fd, pn)) in sc.fld.iter().zip(sc.pan.iter()).skip(2).enumerate() {
            if i >= tk - 2 {
                break;
            }
            let ii = fld_2_var(fd).tousz();
            let va = a.v[ii].v;
            let vp = val_2_form(va, pn.as_str());
            if sc.sum {
                vas.push(va);
            }
            v.push(vp);
        }
        if !sc.yrs.is_empty() {
            let ii = fld_2_var(&sc.fld[tk - 1]).tousz();
            let pn = sc.pan[sc.pan.len() - 1].to_string();
            for y in sc.yrs.iter() {
                if *y < a.vy[ii].len() {
                    let va = a.vy[ii][*y];
                    let vp = val_2_form(va, pn.as_str());
                    if sc.sum {
                        vas.push(va);
                    }
                    v.push(vp);
                }
            }
        }
        if sc.sum {
            vsm = add_2_sum(vsm, vas);
        }
        vv.push(v);
    }
    if sc.sum {
        let mut v = Vec::<String>::new();
        v.push("".to_string());
        v.push("".to_string());
        for (i, va) in vsm.iter().enumerate() {
            let pi = if i + 2 >= sc.pan.len() {
                sc.pan.len() - 1
            } else {
                i + 2
            };
            let pn = sc.pan[pi].clone();
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
}

pub fn ass_var_prv(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string()]];
    let buf = std::fs::read(format!("{dnm}/000-pvrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "P")
}

pub fn ass_var_sub(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    println!("=========== pvid: {}", sc.pvid);
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string(), "1.2".to_string()]];
    //println!("ass var prv sc:{sc:?}");
    let buf = std::fs::read(format!("{dnm}/000-sbrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    /*
    let buf = std::fs::read(format!("{dnm}/000-subm.bin")).unwrap();
    let Ok((subm, _)): Result<(HashMap<String, PeaSub>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    */
    if !sc.pvid.is_empty() {
        assv0 = assv0
            .into_iter()
            .filter(|a| a.pvid == sc.pvid)
            .collect::<_>();
    }
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "S")
    /*
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });
    */
    //println!("ass: {}", assv0.len());
    /*
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        let sbnm = if let Some(psb) = subm.get(&a.sbid) {
            //format!("{} - {} - {}", a.sbid, psb.name, psb.prov)
            format!("{} - {}", a.sbid, psb.name)
        } else {
            a.sbid.to_string()
        };
        v.push(sbnm);
        let mut vas = Vec::<f32>::new();
        for (fd, pn) in sc.fld.iter().zip(sc.pan.iter()).skip(2) {
            let i = fld_2_var(fd).tousz();
            let va = a.v[i].v;
            let vp = val_2_form(va, pn.as_str());
            if sc.sum {
                vas.push(va);
            }
            v.push(vp);
        }
        if sc.sum {
            vsm = add_2_sum(vsm, vas);
        }
        vv.push(v);
    }
    if sc.sum {
        let mut v = Vec::<String>::new();
        v.push("".to_string());
        v.push("".to_string());
        for (i, va) in vsm.iter().enumerate() {
            let pn = sc.pan[i + 2].clone();
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
    */
}

pub fn ass_var_sub_tr(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    //println!("=========== sbid: {}", sc.sbid);
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string(), "1.2".to_string()]];
    let Ok(buf) = std::fs::read(format!("{dnm}/{}-rw4.bin", sc.sbid)) else {
        return dum;
    };
    //println!("ass var prv sc:{sc:?}");
    //let buf = std::fs::read(format!("{dnm}/000-sbrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    if !sc.sbid.is_empty() {
        assv0 = assv0
            .into_iter()
            .filter(|a| a.sbid == sc.sbid && a.v[VarType::NoPeaTr.tousz()].v > 0.0)
            .collect::<_>();
    }
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "T")
    /*
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });
    */
    //println!("ass: {}", assv0.len());
    /*
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        v.push(a.peano.clone());
        let mut vas = Vec::<f32>::new();
        for (fd, pn) in sc.fld.iter().zip(sc.pan.iter()).skip(2) {
            let i = fld_2_var(fd).tousz();
            let va = a.v[i].v;
            let vp = val_2_form(va, pn.as_str());
            if sc.sum {
                vas.push(va);
            }
            v.push(vp);
        }
        if sc.sum {
            vsm = add_2_sum(vsm, vas);
        }
        vv.push(v);
    }
    if sc.sum {
        let mut v = Vec::<String>::new();
        v.push("".to_string());
        v.push("".to_string());
        for (i, va) in vsm.iter().enumerate() {
            let pn = sc.pan[i + 2].clone();
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
    */
}

pub fn ass_var_fd_tr(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    //println!("=========== fdid: {}", sc.fdid);
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string(), "1.2".to_string()]];
    let Ok(buf) = std::fs::read(format!("{dnm}/{}-rw4.bin", &sc.fdid[..3])) else {
        println!(" >>>>>>>> NOFILE {}", &sc.fdid[..3]);
        return dum;
    };
    //println!("ass var prv sc:{sc:?}");
    //let buf = std::fs::read(format!("{dnm}/000-sbrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    if !sc.fdid.is_empty() {
        assv0 = assv0
            .into_iter()
            .filter(|a| {
                a.fdid == sc.fdid
                    && a.v[VarType::NoPeaTr.tousz()].v > 0.0
                    && a.v[VarType::AllNoMeterTr.tousz()].v > 0.0
            })
            .collect::<_>();
    }
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "T")
    /*
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });
    */
    //println!("ass: {}", assv0.len());
    /*
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        v.push(a.peano.clone());
        let mut vas = Vec::<f32>::new();
        for (fd, pn) in sc.fld.iter().zip(sc.pan.iter()).skip(2) {
            let i = fld_2_var(fd).tousz();
            let va = a.v[i].v;
            let vp = val_2_form(va, pn.as_str());
            if sc.sum {
                vas.push(va);
            }
            v.push(vp);
        }
        if sc.sum {
            vsm = add_2_sum(vsm, vas);
        }
        vv.push(v);
    }
    if sc.sum {
        let mut v = Vec::<String>::new();
        v.push("".to_string());
        v.push("".to_string());
        for (i, va) in vsm.iter().enumerate() {
            let pn = sc.pan[i + 2].clone();
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
        */
}

pub fn ass_var_prv_yr(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    let dum = vec![vec!["1.1".to_string()]];
    //println!("ass var prv sc:{sc:?}");
    let dnm = crate::dcl::get_dirnm();
    let buf = std::fs::read(format!("{dnm}/000-pvrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    ass_reorder(&mut assv0, sc);
    //println!("ass: {}", assv0.len());
    let mut ys = vec![0, 1, 2, 3, 12, 13, 14];
    if !sc.yrs.is_empty() {
        ys = sc.yrs.clone();
    }
    println!("years: {:?}", sc.yrs);
    let mut hds = Vec::<String>::new();
    for (i, h) in sc.hds.iter().enumerate() {
        if i > 2 {
            break;
        }
        hds.push(h.clone());
    }
    for i in &ys {
        hds.push(format!("{}", i + 2026));
    }
    sc.hds = hds;

    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    let fd = sc.fld[2].clone();
    let pn = sc.pan[2].clone();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        v.push(a.pvid.clone());
        let i = fld_2_var(&fd).tousz();
        let mut vas = Vec::<f32>::new();
        for y in &ys {
            if *y >= a.vy.len() {
                continue;
            }
            let va = a.vy[i][*y];
            if sc.sum {
                vas.push(va);
            }
            let vp = val_2_form(va, pn.as_str());
            v.push(vp);
        }
        if sc.sum {
            vsm = add_2_sum(vsm, vas);
        }
        vv.push(v);
    }
    if sc.sum {
        let mut v = Vec::<String>::new();
        v.push("".to_string());
        v.push("".to_string());
        for va in &vsm {
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
}

pub fn add_2_sum(mut vsm: Vec<f32>, vas: Vec<f32>) -> Vec<f32> {
    for (i, va) in vas.iter().enumerate() {
        if i >= vsm.len() {
            vsm.push(*va);
        } else {
            vsm[i] += *va;
        }
    }
    vsm
}

//======================================================
pub fn tb1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &mut ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut sc = script_param(pws);

    // table title
    let lst = pws.last().unwrap();
    println!("tb1 {lst}");
    docv = sp(docv, vw)?;
    let lst = modi_tab(lst.as_str());
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .align(AlignmentType::Center);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;

    // table func dispatcher
    let data = match &sc.cmd {
        None => data0(&sc),
        Some(c) => match c.as_str() {
            //"data0" => data0(&sc),
            "diag_to_rows" => diag_to_rows(&sc, vw),
            "ass_var_prv" => ass_var_prv(&mut sc),
            "ass_var_sub" => ass_var_sub(&mut sc),
            "ass_var_sub_tr" => ass_var_sub_tr(&mut sc),
            "ass_var_fd_tr" => ass_var_fd_tr(&mut sc),
            "ass_var_aoj_tr" => ass_var_aoj_tr(&mut sc),
            "ass_var_aoj" => ass_var_aoj(&mut sc),
            "ass_var_prv_yr" => ass_var_prv_yr(&mut sc),
            /*
            "DiagDoc" => diag_to_rows(&sc, vw),
            "AssVarPrv" => ass_var_prv(&sc),
            "AssVarSub" => ass_var_sub(&sc),
            "AssVarSubTr" => ass_var_sub_tr(&sc),
            "AssVarFdTr" => ass_var_fd_tr(&sc),
            "AssVarPrvYr" => ass_var_prv_yr(&mut sc),
            "AssVarAoj" => ass_var_aoj(&sc),
            "AssVarAojTr" => ass_var_aoj_tr(&sc),
            */
            _ => data0(&sc),
        },
    };

    if data.is_empty() {
        return Ok(docv);
    }

    let rw0 = &data[0];

    let mut rows = Vec::<TableRow>::new();
    //==== table header begin
    let mut hdrw = Vec::<TableCell>::new();
    //println!("header #2");
    let mut cwd = 100;
    for c in 0..rw0.len() {
        let mut rr = Run::new();
        let h = if c < sc.hds.len() {
            sc.hds[c].to_string()
        } else {
            if c < sc.fld.len() {
                sc.fld[c].to_string()
            } else {
                "-".to_string()
            }
        };
        rr = rr.add_text(&h);
        rr = rr.fonts(RunFonts::new().cs(STD_FONT));
        rr = rr.size(32);
        let mut pa = Paragraph::new();
        pa = pa.add_run(rr);
        pa = pa.align(AlignmentType::Center);
        let mut ce = TableCell::new();
        ce = ce.add_paragraph(pa);
        ce = ce.shading(Shading::new().fill(TABSHADE));
        if c < sc.wds.len() {
            cwd = sc.wds[c];
            //let w = sc.wds[c];
            //println!("  '{h}' - {w}");
        }
        ce = ce.width(cwd, WidthType::Dxa);
        hdrw.push(ce);
    }
    rows.push(TableRow::new(hdrw));
    //====  table header end

    for rw in data.iter() {
        let mut dtcs = Vec::<TableCell>::new();
        for (c, ce) in rw.iter().enumerate() {
            let mut rr = Run::new();
            rr = rr.add_text(ce);
            //rr = rr.add_text(format!("r:{i}.{j}"));
            rr = rr.fonts(RunFonts::new().cs(STD_FONT));
            rr = rr.size(32);
            let mut pa = Paragraph::new();
            pa = pa.add_run(rr);
            let a = if c < sc.als.len() {
                sc.als[c]
            } else {
                AlignmentType::Center
            };
            pa = pa.align(a);
            pa = pa.indent(Some(CELL_MARGIN), None, Some(CELL_MARGIN), None);
            let mut ce = TableCell::new();
            ce = ce.add_paragraph(pa);
            dtcs.push(ce);
        }
        rows.push(TableRow::new(dtcs));
    }
    let tb = Table::new(rows);
    let tb = tb.align(TableAlignmentType::Center);
    //let tb = tb.indent(DOC_TB_INDENT);
    docv = add_docxv(docv, DocxCompo::Table(tb), vw)?;

    let pa = Paragraph::new().add_run(Run::new());
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

pub fn ar_line_doc(
    mut docv: Vec<DocxCompo>,
    lines: &[String],
    vw: &mut ArViewDocInfo,
    isvw: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut linev = Vec::<Vec<String>>::new();
    for ln in lines.iter() {
        let ln = ln.replace("$name", &vw.vnm);
        let pws = ar_cmd_split(ln.as_str());
        docv = match pws[0].as_str() {
            "h1" => hn(docv, &pws, 1, vw)?,
            "h2" => hn(docv, &pws, 2, vw)?,
            "h3" => hn(docv, &pws, 3, vw)?,
            "h4" => hn(docv, &pws, 4, vw)?,
            "h5" => hn(docv, &pws, 5, vw)?,
            "sp" => sp(docv, vw)?,
            "pg" => sp(docv, vw)?,
            "img1" => img1(docv, &pws, vw)?,
            "img2" => img2(docv, &pws, vw)?,
            "img3" => img3(docv, &pws, vw)?,
            "d1" => dn(docv, &pws, 1, vw)?,
            "d2" => dn(docv, &pws, 2, vw)?,
            "d3" => dn(docv, &pws, 3, vw)?,
            "d4" => dn(docv, &pws, 4, vw)?,
            "d5" => dn(docv, &pws, 5, vw)?,
            "doc1" => dn(docv, &pws, 1, vw)?,
            "doc2" => dn(docv, &pws, 2, vw)?,
            "doc3" => dn(docv, &pws, 3, vw)?,
            "t1" => tb1(docv, &pws, vw)?,
            "v1" if isvw => v1(docv, &pws, vw)?,
            "v2" if isvw => v2(docv, &pws, vw, false)?,
            "v3" if isvw => v2(docv, &pws, vw, true)?,
            "e1" if isvw => e1(docv, &pws, vw)?,
            "e2" if isvw => e2(docv, &pws, vw)?,
            "c1" => c1(docv, &pws, vw)?,
            "c2" => c2(docv, &pws, vw)?,
            "c3" => c3(docv, &pws, vw)?,
            _ => doc0(docv, &pws, vw)?,
        };
        linev.push(pws);
    }
    Ok(docv)
}

pub fn ar_view_elem(
    e: &std::cell::Ref<ArchElem>,
    es: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> (Vec<Diagram>, Vec<Diagram>, Vec<Diagram>) {
    let mut diavw = Vec::<Diagram>::new();
    let mut diaem = Vec::<Diagram>::new();
    let mut diaal = Vec::<Diagram>::new();
    println!("count elem: {}", e.child.len());
    for (i,ch) in e.child.iter().enumerate() {
        let rc = ch.clone();
        let ch = ch.borrow();
        let Some(tp) = ch.attr.get("xsi:type") else {
            continue;
        };
        let mut eid = "".to_string();
        let mut xtp = "".to_string();
        let mut prf = "".to_string();
        let mut vtp = "".to_string();
        let dgtp = match tp.as_str() {
            "archimate:DiagramModelReference" => {
                eid = ch.attr.get("model").unwrap_or(&eid).to_string();
                if let Some(el) = es.get(&eid) {
                    let el = el.borrow();
                    xtp = el.attr.get("xsi:type").unwrap_or(&xtp).to_string();
                    //println!("model view ref {xtp}");
                }
                DiagramType::ViewRef
            }
            "archimate:DiagramObject" => {
                eid = ch.attr.get("archimateElement").unwrap_or(&eid).to_string();
                if let Some(el) = es.get(&eid) {
                    let el = el.borrow();
                    let nm = el.attr.get("name").unwrap_or(&xtp).to_string();
                    let id = el.attr.get("id").unwrap_or(&xtp).to_string();
                    let cid= ch.attr.get("id").unwrap_or(&xtp).to_string();
                    println!("  {i}.elem name: {nm} id:{id} eid={eid} cid:{cid}");
                    xtp = el.attr.get("xsi:type").unwrap_or(&xtp).to_string();
                    vtp = el.attr.get("viewpoint").unwrap_or(&vtp).to_string();
                    let mut prfid = "".to_string();
                    prfid = el.attr.get("profiles").unwrap_or(&prfid).to_string();
                    //println!("diagram ref: {xtp}");
                    //println!("============ PROF ref: {prfid}");
                    if let Some(prof) = es.get(&prfid) {
                        let prof = prof.borrow();
                        let mut pnm = "".to_string();
                        pnm = prof.attr.get("name").unwrap_or(&pnm).to_string();
                        //println!("   >>>>>>>>======  profile name : {pnm}");
                        prf = pnm;
                    }
                }
                DiagramType::ElemRef
            }
            "archimate:Note" => DiagramType::NoteRef,
            d => {
                println!("Uncovered Diagram ..1: {d}");
                DiagramType::None
            }
        };
        if let DiagramType::None = dgtp {
            println!("ERROR DIAGRAM");
            continue;
        }
        let Ok(bnd) = get_elem_bounds(&ch) else {
            return (diavw, diaem, diaal);
        };
        let dia = Diagram {
            eid,
            dgtp,
            bnd,
            rc,
            xtp,
            prf,
            vtp,
        };
        match dia.dgtp {
            DiagramType::ViewRef => {
                diavw.push(dia.clone());
                diaal.push(dia);
            }
            DiagramType::ElemRef => {
                diaem.push(dia.clone());
                diaal.push(dia);
            }
            _ => {}
        }
    }
    (diavw, diaem, diaal)
}

pub fn ar_view_ids(
    mut vids: Vec<String>,
    vid: &str,
    es: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> Result<Vec<String>, Box<dyn Error>> {
    if let Some(e) = es.get(vid) {
        vids.push(vid.to_string());
        let e = e.borrow();
        let (diavw, _diaem, _diaal) = ar_view_elem(&e, es);
        for e in &diavw {
            vids = ar_view_ids(vids, &e.eid, es)?;
        }
        Ok(vids)
    } else {
        Err(format!("No eid {vid}").into())
    }
}

pub fn diag_rearange(diavw: &Vec<Diagram>) -> Vec<Vec<Diagram>> {
    let mut disv: Vec<Vec<_>> = vec![];
    if !diavw.is_empty() {
        let mut bndv = vec![diavw[0].bnd.clone()];
        disv = vec![vec![]];
        for di in diavw {
            let mut cn = 0;
            for (i, bnd) in bndv.iter_mut().enumerate() {
                if di.bnd.x0 <= bnd.x1 && di.bnd.x1 >= bnd.x0 {
                    bnd.x0 = di.bnd.x0.min(bnd.x0);
                    bnd.x1 = di.bnd.x1.max(bnd.x1);
                    cn += 1;
                    disv[i].push(di.clone());
                }
            }
            if cn == 0 {
                bndv.push(di.bnd.clone());
                disv.push(vec![di.clone()]);
            }
        }
    }
    disv.sort_by(|a, b| a[0].bnd.x0.cmp(&b[0].bnd.x0));
    //for (i, dis) in disv.iter_mut().enumerate() {
    for dis in disv.iter_mut() {
        //println!("== {i}-{}", dis.len());
        dis.sort_by(|a, b| a.bnd.y0.cmp(&b.bnd.y0));
    }
    //println!("bndv: {}", bndv.len());
    disv
}

pub fn ar_gen_doc(
    docv: Vec<DocxCompo>,
    vid: &str,
    es: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut docv = docv;
    println!("ar gen doc vid: {vid}");
    if let Some(e) = es.get(vid) {
        let e = e.borrow();

        let doc = get_elem_doc(&e);
        let lines = txt_lines(doc);
        let (diavw, diaem, diaal) = ar_view_elem(&e, es);
        let vnm = if let Some(n) = e.attr.get("name") {
            n.to_string()
        } else {
            "?".to_string()
        };
        println!("name: {vnm}");

        let nms = e.names.to_string();
        let edia = diag_rearange(&diaem);
        let vdia = diag_rearange(&diavw);
        let adia = diag_rearange(&diaal);
        println!("edia: {} vdia: {}", edia.len(), vdia.len());
        let mut vw = ArViewDocInfo {
            vid,
            vnm,
            nms,
            es,
            vdia: &vdia,
            edia: &edia,
            adia: &adia,
        };
        let l1 = docv.len();
        docv = ar_line_doc(docv, &lines, &mut vw, true)?;
        let l2 = docv.len();
        if l2 > l1 {
            docv = end_ref(docv, &vw)?;
        }
        Ok(docv)
    } else {
        Err(format!("No eid {vid}").into())
    }
}

pub fn get_elem_prop(el: &std::cell::Ref<ArchElem>, p_key: &str) -> Vec<String> {
    let mut props = Vec::<String>::new();
    for ch in &el.child {
        let ch = ch.borrow();
        if ch.elem == "property"
            && let Some(ky) = ch.attr.get("key")
            && ky == p_key
            && let Some(va) = ch.attr.get("value")
        {
            props.push(va.to_string());
        }
    }
    props
}

pub fn get_elem_doc(el: &std::cell::Ref<ArchElem>) -> String {
    for ch in &el.child {
        let ch = ch.borrow();
        if ch.elem == "documentation" {
            let tx = ch.text.to_string();
            return tx;
        }
    }
    String::new()
}

pub fn get_elem_bounds(el: &std::cell::Ref<ArchElem>) -> Result<Bound, Box<dyn Error>> {
    for ch in &el.child {
        let ch = ch.borrow();
        if ch.elem == "bounds" {
            let x0 = ch.attr.get("x").unwrap().parse::<i32>()?;
            let y0 = ch.attr.get("y").unwrap().parse::<i32>()?;
            let x1 = x0 + ch.attr.get("width").unwrap().parse::<i32>()?;
            let y1 = y0 + ch.attr.get("height").unwrap().parse::<i32>()?;
            return Ok(Bound { x0, y0, x1, y1 });
        }
    }
    Err("Error".into())
}

fn archi_extract(indir: &str, outdir: &str) -> Result<(), Box<dyn Error>> {
    let parch = std::path::Path::new(indir);
    let farch = fs::File::open(parch).unwrap();
    let mut arch = zip::ZipArchive::new(farch).unwrap();
    //println!("archi {}", arch.len());

    let img_dir = format!("{}/images/", outdir);
    fs::create_dir_all(img_dir).unwrap();
    let mut imcn = 0;
    for i in 0..arch.len() {
        let mut file = arch.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        if file.is_dir() {
            let dir = format!("{}/{}", outdir, outpath.display());
            fs::create_dir_all(&dir).unwrap();
            //println!("dir: {dir}");
        } else {
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                //println!("p:{p:?}");
            }
            let fnm = format!("{}/{}", outdir, outpath.display());
            if !fnm.ends_with(".png") && !fnm.ends_with(".jpg") {
                println!("save: {fnm}");
                let mut outfile = fs::File::create(&fnm).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
            imcn += 1;
        }
    }
    println!("==== image cnt: {imcn}");
    Ok(())
}

//fn vec_j(v: &Vec<String>, i1: usize, i2: usize) -> String {
pub fn vec_ij(v: &[String], i: usize, j: usize) -> String {
    v.iter()
        .enumerate()
        .filter(|(ii, _)| *ii >= i && *ii <= j)
        .map(|(_, d)| d.to_owned())
        .collect::<Vec<String>>()
        .join("/")
}

#[derive(Debug, Clone, Default)]
pub struct ArchElem {
    pub elem: String,
    pub text: String,
    pub attr: HashMap<String, String>,
    pub names: String,
    pub child: Vec<Rc<RefCell<ArchElem>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ArchiModel {
    elem: String,
    text: String,
    attr: HashMap<String, String>,
    paths: String,
    names: String,
    child: Vec<ArchiModel>,
}

pub const XTG_MODEL: &[u8] = b"model";
pub const XTG_FOLDER: &[u8] = b"folder";
pub const XTG_PROFILE: &[u8] = b"profile";
pub const XTG_ELEMENT: &[u8] = b"element";
pub const XTG_PROPERTY: &[u8] = b"property";
pub const XTG_BOUNDS: &[u8] = b"bounds";
pub const XTG_SOURCE_CONNECNTION: &[u8] = b"sourceConnection";
pub const XTG_BENDPOINT: &[u8] = b"bendpoint";
pub const XTG_FEATURE: &[u8] = b"feature";
pub const XTG_CHILD: &[u8] = b"child";
pub const XTG_CONTENT: &[u8] = b"content";
pub const XTG_HINT_CONTENT: &[u8] = b"hintContent";
pub const XTG_DOCUMENTATION: &[u8] = b"documentation";

use quick_xml::events::BytesStart;

impl ArchElem {
    pub fn new(e: &BytesStart, namev: &[String]) -> Self {
        ArchElem {
            elem: String::from_utf8(e.local_name().as_ref().to_vec()).unwrap(),
            attr: attr_map(e),
            //paths: vec_ij(pathv, 1, 10),
            names: vec_ij(namev, 1, 10),
            ..Default::default()
        }
    }
}

fn archi_ana1() -> Result<(ArchElem, ArchElem), Box<dyn Error>> {
    let fmod = "archi/model.xml";
    if let Ok(mut xrd) = XmlReader::from_file(fmod) {
        //println!("XML OPEN {fmod}");
        let mut xbuf = Vec::new();
        let mut pathv = Vec::<String>::new();
        let mut namev = Vec::<String>::new();
        let mut ar_elem = ArchElem {
            elem: String::from("element"),
            ..Default::default()
        };
        let mut ar_fold = ArchElem {
            elem: String::from("folder"),
            ..Default::default()
        };
        let mut ar_stack = Vec::<Rc<RefCell<ArchElem>>>::new();
        loop {
            match xrd.read_event_into(&mut xbuf) {
                Ok(Event::Eof) => break,
                Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                    XTG_MODEL => {
                        ar_elem.attr = attr_map(&e);
                    }
                    XTG_FOLDER => {}
                    XTG_ELEMENT | XTG_PROFILE => {
                        let el = ArchElem::new(&e, &namev);
                        let ee = Rc::new(RefCell::new(el));
                        ar_elem.child.push(ee);
                        if ar_fold.child.is_empty() {
                            println!("EMPTY FOLDER");
                        }
                    }
                    XTG_PROPERTY
                    | XTG_BOUNDS
                    | XTG_SOURCE_CONNECNTION
                    | XTG_BENDPOINT
                    | XTG_FEATURE
                    | XTG_CHILD
                    | XTG_CONTENT
                    | XTG_HINT_CONTENT
                    | XTG_DOCUMENTATION => {
                        let el = ArchElem::new(&e, &namev);
                        if !ar_stack.is_empty() {
                            let ii = ar_stack.len() - 1;
                            let ar = &mut ar_stack[ii];
                            let mut ar = ar.borrow_mut();
                            let ee1 = Rc::new(RefCell::new(el));
                            ar.child.push(ee1);
                        } else {
                            println!("E2: {}", el.elem);
                        }
                    }
                    e => {
                        println!(
                            " error empty element - {}",
                            String::from_utf8(e.to_vec()).unwrap()
                        );
                    }
                },
                Ok(Event::Start(e)) => {
                    let e_name = String::from_utf8(e.local_name().as_ref().to_vec()).unwrap();
                    let a_name = attr(&e, b"name");
                    pathv.push(e_name.clone());
                    if let Some(a) = a_name {
                        namev.push(a.clone());
                    } else {
                        //println!("NO NAME {namev:?}");
                        namev.push("?".to_string());
                    }
                    match e.local_name().as_ref() {
                        XTG_MODEL => {
                            ar_elem.attr = attr_map(&e);
                        }
                        XTG_FOLDER => {
                            let el = ArchElem::new(&e, &namev);
                            let ee1 = Rc::new(RefCell::new(el));
                            let ee2 = ee1.clone();
                            let ee3 = ee1.clone();
                            if !ar_stack.is_empty() {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                ar.child.push(ee3);
                            }
                            ar_stack.push(ee1);
                            ar_fold.child.push(ee2);
                        }
                        XTG_ELEMENT => {
                            let el = ArchElem::new(&e, &namev);
                            let ee1 = Rc::new(RefCell::new(el));
                            let ee2 = ee1.clone();
                            let ee3 = ee1.clone();
                            ar_elem.child.push(ee1);
                            if ar_stack.is_empty() {
                                println!("ERROR4: ");
                            } else {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                ar.child.push(ee3);
                            }
                            ar_stack.push(ee2);
                        }
                        XTG_CHILD
                        | XTG_CONTENT
                        | XTG_HINT_CONTENT
                        | XTG_SOURCE_CONNECNTION
                        | XTG_DOCUMENTATION => {
                            let el = ArchElem::new(&e, &namev);
                            //let id = "".to_string();
                            //let id = el.attr.get("id").unwrap_or(&id).to_string();
                            let elm = el.elem.to_string();
                            let ee1 = Rc::new(RefCell::new(el));
                            let ee2 = ee1.clone();
                            if ar_stack.is_empty() {
                                println!("E3: {} {:?}", elm, namev);
                            } else {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                ar.child.push(ee1);
                            }
                            ar_stack.push(ee2);
                        }
                        e => {
                            println!(" error - {}", String::from_utf8(e.to_vec()).unwrap());
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    pathv.pop();
                    namev.pop();
                    match e.local_name().as_ref() {
                        XTG_MODEL => {}
                        XTG_FOLDER => {
                            ar_stack.pop();
                        }
                        XTG_ELEMENT => {
                            ar_stack.pop();
                        }
                        XTG_CHILD
                        | XTG_CONTENT
                        | XTG_HINT_CONTENT
                        | XTG_SOURCE_CONNECNTION
                        | XTG_DOCUMENTATION => {
                            ar_stack.pop();
                        }
                        e => {
                            println!("ERROR 3 : {}", String::from_utf8(e.to_vec()).unwrap());
                        }
                    }
                }
                Ok(Event::Text(tx)) => {
                    let tx = String::from_utf8(tx.to_vec()).unwrap_or_default();
                    let tx = tx.trim();
                    if !tx.is_empty() {
                        {
                            let ii = ar_stack.len() - 1;
                            let ar = &mut ar_stack[ii];
                            let mut ar = ar.borrow_mut();
                            //ar.text = tx.to_string();
                            ar.text = format!("{}{}", ar.text, tx);
                            //println!("   DOC TEXT e:'{}' - t:''", ar.elem);
                        }
                    }
                }
                Ok(Event::GeneralRef(ge)) => {
                    let ge = String::from_utf8(ge.to_vec()).unwrap_or_default();
                    let tx = match ge.as_str() {
                        "#xD" => "\n".to_string(),
                        "lt" => "<".to_string(),
                        "gt" => "<".to_string(),
                        "amp" => "&".to_string(),
                        "quot" => "\"".to_string(),
                        "apos" => "\'".to_string(),
                        e => {
                            println!("{e:?}");
                            e.to_string()
                        }
                    };
                    if !tx.is_empty() {
                        {
                            let ii = ar_stack.len() - 1;
                            let ar = &mut ar_stack[ii];
                            let mut ar = ar.borrow_mut();
                            //ar.text = tx.to_string();
                            ar.text = format!("{}{}", ar.text, tx);
                            //println!("   DOC TEXT e:'{}' - t:''", ar.elem);
                        }
                    }
                    //let ge = ge.borrow().clone();
                    //println!("========= GeneralRef 2 : {:?}", x);
                    //println!("========= GeneralRef : {:?}", ge);
                }
                Ok(Event::Decl(_de)) => {
                    //println!("========= Decl : {:?}", _de);
                }
                e => {
                    println!("========= ERROR 2 : {:?}", e);
                }
            }
            xbuf.clear();
        } // XML loop
          /*
          let ar_ee = Rc::new(RefCell::new(ar_elem));
          let ar_model = arch_elem_to_model(&ar_ee)?;
          let ar_ee = Rc::new(RefCell::new(ar_fold));
          let ar_folder = arch_elem_to_model(&ar_ee)?;
          return Ok((ar_model, ar_folder));
          */
        return Ok((ar_elem, ar_fold));
    } // open XML file
    Err("Failure".into())
}

#[derive(Debug)]
pub enum DocxCompo {
    Paragraph(Paragraph),
    Style(Style),
    Header(Header),
    Footer(Footer),
    Table(Table),
    ToC(TableOfContents),
}

pub const DOCX_PNTS: i32 = 9525;

pub fn write_docx(
    vdoc: Vec<DocxCompo>,
    pin: &str,
    pout: &str,
    tdir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut docx = Docx::new();
    let fst_hd = Header::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(" ")));
    let mag = PageMargin {
        top: 1000,
        left: 800,
        bottom: 1000,
        right: 1000,
        header: 500,
        footer: 500,
        gutter: 0,
    };
    docx = docx.page_margin(mag);
    for dc in vdoc {
        match dc {
            DocxCompo::Header(h) => {
                docx = docx.header(h);
            }
            DocxCompo::Footer(f) => {
                docx = docx.footer(f);
            }
            DocxCompo::Style(s) => {
                docx = docx.add_style(s);
            }
            DocxCompo::Paragraph(p) => {
                docx = docx.add_paragraph(p);
            }
            DocxCompo::Table(p) => {
                docx = docx.add_table(p);
            }
            DocxCompo::ToC(t) => {
                docx = docx.add_table_of_contents(t);
            }
        }
    }
    docx = docx.first_header(fst_hd);

    let pin = std::path::Path::new(pin);
    let pin1 = std::path::Path::new(pin);
    let pin2 = std::fs::File::create(pin1).unwrap();
    let _ = docx.build().pack(pin2);

    let pout = std::path::Path::new(pout);
    //println!("pin:{pin:?} pout:{pout:?}");

    let fout = std::fs::File::create(pout).unwrap();
    let mut zout = zip::ZipWriter::new(fout);
    let file = fs::File::open(pin).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        if file.is_dir() {
            let dir = format!("{}", outpath.display());
            let dir0 = format!("{}/{}", tdir, outpath.display());
            //println!("dir0:{dir0}");
            fs::create_dir_all(&dir0).unwrap();
            zout.add_directory(&dir, SimpleFileOptions::default())
                .expect("?");
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    //fs::create_dir_all(p).unwrap();
                }
            }
            let fnm = format!("{}", outpath.display());
            let fnm0 = format!("{}/{}", tdir, outpath.display());

            let mut outfile = fs::File::create(&fnm0).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();

            let mut f = std::fs::File::open(&fnm0).expect("no file found");
            let metadata = std::fs::metadata(&fnm0).expect("unable to read metadata");
            let mut buff = vec![0; metadata.len() as usize];
            f.read_exact(&mut buff).expect("buffer overflow");
            //if fnm == "word/document.xml" {
            if ["word/document.xml", "word/header1", "word/footer1"].contains(&fnm.as_str()) {
                let buf2 = String::from_utf8(buff.clone()).unwrap();
                let buf2 = buf2.replace("<w:rFonts", "<w:cs/><w:rFonts");
                /*
                let buf2 = buf2.replace("allowOverlap=\"0\"", "allowOverlap=\"1\"");
                let buf2 = buf2.replace("behindDoc=\"0\"", "behindDoc=\"1\"");
                let buf2 = buf2.replace(
                    "<wp:positionH relativeFrom=\"margin\"><wp:posOffset>0</wp:posOffset>",
                    "<wp:positionH relativeFrom=\"margin\"><wp:posOffset>-3175</wp:posOffset>",
                );
                let buf2 = buf2.replace(
                    "<wp:positionV relativeFrom=\"margin\"><wp:posOffset>0</wp:posOffset>",
                    "<wp:positionV relativeFrom=\"margin\"><wp:posOffset>3175</wp:posOffset>",
                );
                let buf2 = buf2.replace(
                    "<w:rPr><w:sz w:val=\"60\" /><w:szCs w:val=\"60\" /><w:cs/><w:rFonts w:cs=\"TH Sarabun New\" /></w:rPr><w:t xml:space=\"preserve\">",
                    "<w:rPr><w:sz w:val=\"60\" /><w:szCs w:val=\"60\" /><w:cs/><w:rFonts w:cs=\"TH Sarabun New\" /></w:rPr><w:t>"
                );
                */
                //println!("fnm:{fnm} 1:{} 2:{}", buff.len(), buf2.len());
                buff = buf2.as_bytes().to_vec();
            }
            let options = SimpleFileOptions::default();
            zout.start_file(fnm, options).expect("?");
            zout.write_all(&buff).expect("?");
        }
    }
    zout.finish().expect("?");
    Ok(())
}

pub fn attr_map(e: &quick_xml::events::BytesStart) -> HashMap<String, String> {
    let mut am = HashMap::<String, String>::new();
    for at in e.attributes().flatten() {
        let key = String::from_utf8(at.key.as_ref().to_vec()).unwrap_or_default();
        let val = match at.value {
            Cow::Borrowed(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
            Cow::Owned(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
        };
        am.insert(key, val);
    }
    am
}

pub fn attr(e: &quick_xml::events::BytesStart, atnm: &[u8]) -> Option<String> {
    for at in e.attributes().flatten() {
        match at.key.as_ref() {
            ky if ky == atnm => {
                let va = match at.value {
                    Cow::Borrowed(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                    Cow::Owned(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                };
                return Some(va);
            }
            _ => {}
        }
    }
    None
}

/*
pub fn attr0(
    e: &quick_xml::events::BytesStart,
) -> (Option<String>, Option<String>, Option<String>) {
    let (mut tp, mut nm, mut id) = (None, None, None);
    for at in e.attributes().flatten() {
        match at.key.as_ref() {
            ky if ky == b"xsi:type" => {
                tp = Some(match at.value {
                    Cow::Borrowed(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                    Cow::Owned(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                });
            }
            ky if ky == b"name" => {
                nm = Some(match at.value {
                    Cow::Borrowed(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                    Cow::Owned(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                });
            }
            ky if ky == b"id" => {
                id = Some(match at.value {
                    Cow::Borrowed(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                    Cow::Owned(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
                });
            }
            _ => {}
        }
    }
    (tp, nm, id)
}

pub fn docx_adj(pin: &str, fout: &str) {
    let tdir = "temp";
    let fout = std::path::Path::new(fout);
    let fout = std::fs::File::create(fout).unwrap();
    let mut zout = zip::ZipWriter::new(fout);
    let file = std::fs::File::open(pin).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        if file.is_dir() {
            let dir = format!("{}", outpath.display());
            let dir0 = format!("{}/{}", tdir, outpath.display());
            //println!("dir0:{dir0}");
            std::fs::create_dir_all(&dir0).unwrap();
            zout.add_directory(&dir, SimpleFileOptions::default())
                .expect("?");
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    //fs::create_dir_all(p).unwrap();
                }
            }
            let fnm = format!("{}", outpath.display());
            let fnm0 = format!("{}/{}", tdir, outpath.display());

            let mut outfile = std::fs::File::create(&fnm0).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();

            let mut f = std::fs::File::open(&fnm0).expect("no file found");
            let metadata = std::fs::metadata(&fnm0).expect("unable to read metadata");
            let mut buff = vec![0; metadata.len() as usize];
            f.read_exact(&mut buff).expect("buffer overflow");
            if fnm == "word/document.xml" {
                let buf2 = String::from_utf8(buff.clone()).unwrap();
                let buf2 = buf2.replace("<w:rFonts", "<w:cs/><w:rFonts");
                //println!("fnm:{fnm} 1:{} 2:{}", buff.len(), buf2.len());
                buff = buf2.as_bytes().to_vec();
            }
            let options = SimpleFileOptions::default();
            zout.start_file(fnm, options).expect("?");
            zout.write_all(&buff).expect("?");
        }
    }
    zout.finish().expect("?");
}

const HD1SZ: usize = 48;
const HD2SZ: usize = 40;
const HD3SZ: usize = 36;
const TX_SZ: usize = 36;
const THFONT: &str = "TH Sarabun New";

#[allow(dead_code)]
pub fn para_h1(tx: &str) -> Paragraph {
    para(tx, "Header 1", HD1SZ, false)
}
#[allow(dead_code)]
pub fn para_h2(tx: &str) -> Paragraph {
    para(tx, "Header 2", HD2SZ, false)
}
#[allow(dead_code)]
pub fn para_h3(tx: &str) -> Paragraph {
    para(tx, "Header 3", HD3SZ, false)
}
#[allow(dead_code)]
pub fn para_nm(tx: &str) -> Paragraph {
    para(tx, "Normal", TX_SZ, false)
}
#[allow(dead_code)]
pub fn para_n1(tx: &str) -> Paragraph {
    para1(tx, "Normal", TX_SZ, false, 750)
}

#[allow(dead_code)]
pub fn page_h1(tx: &str) -> Paragraph {
    para(tx, "Header 1", HD1SZ, true)
}
#[allow(dead_code)]
pub fn page_h2(tx: &str) -> Paragraph {
    para(tx, "Header 2", HD2SZ, true)
}
#[allow(dead_code)]
pub fn page_h3(tx: &str) -> Paragraph {
    para(tx, "Header 3", HD3SZ, true)
}

pub fn para1(tx: &str, stl: &str, sz: usize, pg: bool, ind: i32) -> Paragraph {
    Paragraph::new()
        .add_run(
            Run::new()
                .add_text(tx)
                .size(sz)
                .fonts(RunFonts::new().cs(THFONT)),
        )
        .style(stl)
        .page_break_before(pg)
        .indent(
            None,
            Some(docx_rs::SpecialIndentType::FirstLine(ind)),
            None,
            None,
        )
    //.indent(Some(ind), None, None, None)
}

pub fn para(tx: &str, stl: &str, sz: usize, pg: bool) -> Paragraph {
    Paragraph::new()
        .add_run(
            Run::new()
                .add_text(tx)
                .size(sz)
                .fonts(RunFonts::new().cs(THFONT)),
        )
        .style(stl)
        .page_break_before(pg)
}
*/
/*
#[derive(Debug, Clone)]
pub struct DiagramGroup<'a> {
    bnd: Bound,
    diags: Vec<&'a Diagram>,
}

pub fn archi1() -> Result<(), Box<dyn std::error::Error>> {
    let indir = "/mnt/c/Users/choom/Documents/wk33/peasg/archi/SG-admin-2025-02-01.archimate";
    archi_extract(indir, "archi")?;
    let (ar_elem, _ar_fold) = archi_ana1()?;
    println!("elm:{} fold:{}", ar_elem.child.len(), _ar_fold.child.len());
    for (i, e) in ar_elem.child.iter().enumerate() {
        let e = e.borrow();
        let tp = e.attr.get("xsi:type");
        if e.text.is_empty() {
            continue;
        }
        println!(
            "{i} e:'{}' t:'{}' c:{} {:?}",
            e.elem,
            e.text,
            e.child.len(),
            tp,
        );
    }

    let mut wk = ElemWork::default();
    let ar_elem = Rc::new(RefCell::new(ar_elem));
    wk.xsi_type(&ar_elem);

    wk.find_bbro(&ar_elem);
    for (n, _tx, nm, el) in &wk.repo_v {
        let el = el.borrow_mut();
        let nn = el.child.len();
        println!("{}-{}-{}", n, nn, nm);
        for ch in &el.child {
            let ch = ch.borrow_mut();
            println!("  {ch:?}");
        }
    }
    //println!("model ID:{}", ar_elem.attr.get("id").unwrap());
    let vdoc = docx_cont1();
    write_docx(vdoc, "org.docx", "mod.docx", "temp")?;

    Ok(())
}
*/

/*

fn arch_elem_to_model(el: &Rc<RefCell<ArchElem>>) -> Result<ArchiModel, Box<dyn Error>> {
    let el = el.borrow_mut();
    let mut md0 = ArchiModel {
        elem: el.elem.clone(),
        text: el.text.clone(),
        attr: el.attr.clone(),
        paths: el.paths.clone(),
        names: el.names.clone(),
        ..Default::default()
    };
    for ch in &el.child {
        let md1 = arch_elem_to_model(ch)?;
        md0.child.push(md1);
    }
    Ok(md0)
}

fn docx_cont1() -> Vec<DocxCompo> {
    let style1 = Style::new("Heading1", StyleType::Paragraph).name("Heading 1");
    let style2 = Style::new("Heading2", StyleType::Paragraph).name("Heading 2");
    let style3 = Style::new("Heading3", StyleType::Paragraph).name("Heading 3");

    let h1 = Paragraph::new()
        .add_run(
            Run::new()
                .add_text("หัวเรื่องที่๑")
                .size(48)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .style("Heading1")
        .page_break_before(true);

    let h2 = Paragraph::new()
        .add_run(
            Run::new()
                .add_text("หัวเรื่องที่2")
                .size(40)
                .fonts(RunFonts::new().cs("TH Sarabun New")),
        )
        .style("Heading2")
        .page_break_before(true);

    let h3 = Paragraph::new()
        .add_run(
            Run::new()
                .add_text("หัวเรื่อง๓")
                .fonts(RunFonts::new().cs("TH Sarabun New"))
                .size(32),
        )
        .style("Heading3")
        .page_break_before(true);

    let c1 = Paragraph::new().add_run(
        Run::new()
            .add_text("๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑")
            .add_text("๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑")
            .add_text("๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑๑")
            .size(24)
            .fonts(RunFonts::new().cs("TH Sarabun New")),
    ).indent(Some(840),None,None,None);

    let c2 = Paragraph::new().add_run(
        Run::new()
            .add_text("๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒๒")
            .size(24)
            .fonts(RunFonts::new().cs("TH Sarabun New")),
    ).indent(None,Some(SpecialIndentType::FirstLine(720)),None,None);

    let c3 = Paragraph::new().add_run(
        Run::new()
            .add_text("๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓๓")
            .size(24)
            .fonts(RunFonts::new().cs("TH Sarabun New")),
    ).indent(Some(1560),Some(SpecialIndentType::Hanging(720)),None,None);

    let header = Header::new().add_paragraph(
        Paragraph::new().add_run(
            Run::new()
                .add_text("เฮดเดอร์")
                .fonts(RunFonts::new().cs("TH Sarabun New"))
                .size(24),
        ),
    );

    let mut img = std::fs::File::open("./images/bbro1.jpg").unwrap();
    let mut buf = Vec::new();
    let _ = img.read_to_end(&mut buf).unwrap();
    let pic = Pic::new(&buf).size(320 * 9525, 240 * 9525);
    let ppic = Paragraph::new().add_run(Run::new().add_text("🐱").add_image(pic.clone()));

    //let first_header =
    //    Header::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("First")));

    let footer = Footer::new().add_paragraph(
        Paragraph::new().add_run(
            Run::new()
                .add_text("=================== ฟุตเตอร์")
                .fonts(RunFonts::new().cs("TH Sarabun New"))
                .size(24),
        ),
    );

    let mut vdoc = Vec::<DocxCompo>::new();

    vdoc.push(DocxCompo::Header(header));
    vdoc.push(DocxCompo::Footer(footer));
    vdoc.push(DocxCompo::Style(style1));
    vdoc.push(DocxCompo::Style(style2));
    vdoc.push(DocxCompo::Style(style3));
    vdoc.push(DocxCompo::Paragraph(h1));
    vdoc.push(DocxCompo::Paragraph(c1));
    vdoc.push(DocxCompo::Paragraph(ppic));
    vdoc.push(DocxCompo::Paragraph(h2));
    vdoc.push(DocxCompo::Paragraph(c2));
    vdoc.push(DocxCompo::Paragraph(h3));
    vdoc.push(DocxCompo::Paragraph(c3));

    vdoc
}

//use crate::prc41::SubCalc;
#[derive(Debug, Clone, Default)]
struct ElemWork {
    er_cnt: u32,
    type_cnt: HashMap<String, u32>,
    chtp_cnt: HashMap<String, u32>,
    todo_cnt: HashMap<String, u32>,
    // repo, tx, name, elem
    repo_v: Vec<(String, String, String, Rc<RefCell<ArchElem>>)>,
}
impl ElemWork {
    //fn find_repo(&mut self, el: &Rc<RefCell<ArchElem>>) -> Option<&Rc<RefCell<ArchElem>>> {
    fn doc_text(els: &[Rc<RefCell<ArchElem>>]) -> String {
        for ch in els {
            let ch = ch.borrow_mut();
            if ch.elem == "documentation" {
                let tx = ch.text.replace("&#xD;", "");
                return tx;
                //println!("  {cn}.{} '{}", ch.elem, tx);
            }
        }
        String::new()
    }
    fn find_bbro(&mut self, el0: &Rc<RefCell<ArchElem>>) {
        let re = regex::Regex::new(r"###BBRO.*###(\w+)").unwrap();
        //let re = regex::Regex::new(r"###BBRO.*###(?<name>\w+)").unwrap();
        let mut rs = None;
        {
            let el = el0.borrow_mut();
            if el.elem.as_bytes() == XTG_ELEMENT
                && let (Some(tp), Some(nm)) = (el.attr.get("xsi:type"), el.attr.get("name"))
                && tp == "archimate:ArchimateDiagramModel"
            {
                let tx = ElemWork::doc_text(&el.child);
                if let Some(bbr) = re.captures(&tx) {
                    //let bb2 = &bbr["name"];
                    let bb2 = &bbr[1];
                    let bb2 = bb2.to_string();
                    let tx2 = tx.to_string();
                    println!("view {} '{}' {bb2}", nm, tx);
                    rs = Some((bb2, tx2, nm.clone()));
                    //self.repo_v.push((bb2, tx2, el0.clone()));
                }
            }
        }
        if let Some((rp, tx, nm)) = rs {
            self.repo_v.push((rp, tx, nm, el0.clone()));
        }
        {
            let el = el0.borrow_mut();
            for ch in &el.child {
                self.find_bbro(ch);
            }
        }
    }

    fn xsi_type(&mut self, el: &Rc<RefCell<ArchElem>>) {
        let el = el.borrow_mut();
        //println!("type {:?}", el.attr);
        match el.elem.as_bytes() {
            XTG_ELEMENT => {
                if let Some(tp) = el.attr.get("xsi:type") {
                    if let Some(cn) = self.type_cnt.get_mut(tp) {
                        *cn += 1;
                    } else {
                        self.type_cnt.insert(tp.clone(), 1);
                    }
                } else {
                    self.er_cnt += 1;
                    //println!("## error {}.{:?}", self.er_cnt, el.attr);
                }
            }
            XTG_CHILD => {
                if let Some(tp) = el.attr.get("xsi:type") {
                    if let Some(cn) = self.chtp_cnt.get_mut(tp) {
                        *cn += 1;
                    } else {
                        self.chtp_cnt.insert(tp.clone(), 1);
                    }
                } else {
                    println!("error2 {}.{:?}", self.er_cnt, el.attr);
                }
            }
            XTG_PROFILE => {}
            e => {
                let elm = String::from_utf8(e.to_vec()).unwrap();
                if let Some(cn) = self.todo_cnt.get_mut(&elm) {
                    *cn += 1;
                } else {
                    self.todo_cnt.insert(elm, 1);
                }
            }
        }
        for ch in &el.child {
            self.xsi_type(ch);
        }
    }
}


pub fn tb2_gen(sbca: &SubCalc) -> Vec<Vec<String>> {
    vec![
        vec![
            "จำนวนมิเตอร์เฟส เอ".to_string(),
            format!("{} ตัว", sbca.mt_ph_a.separate_with_commas()),
        ],
        vec![
            "จำนวนมิเตอร์เฟส บี".to_string(),
            format!("{} ตัว", sbca.mt_ph_b.separate_with_commas()),
        ],
        vec![
            "จำนวนมิเตอร์เฟส ซี".to_string(),
            format!("{} ตัว", sbca.mt_ph_c.separate_with_commas()),
        ],
        vec![
            "จำนวนมิเตอร์หนึ่งเฟส".to_string(),
            format!("{} ตัว", sbca.mt_1_ph.separate_with_commas()),
        ],
        vec![
            "จำนวนมิเตอร์สามเฟส".to_string(),
            format!("{} ตัว", sbca.mt_3_ph.separate_with_commas()),
        ],
        vec![
            "หน่วยการใช้ไฟฟ้าทั้งปี".to_string(),
            format!("{} MWh", (sbca.eg_sm as u32).separate_with_commas()),
        ],
        vec![
            "หน่วยการใช้ไฟฟ้าเฟสเอทั้งปี".to_string(),
            format!("{} MWh", (sbca.eg_a as u32).separate_with_commas()),
        ],
        vec![
            "หน่วยการใช้ไฟฟ้าเฟสบีทั้งปี".to_string(),
            format!("{} MWh", (sbca.eg_b as u32).separate_with_commas()),
        ],
        vec![
            "หน่วยการใช้ไฟฟ้าเฟสซีทั้งปี".to_string(),
            format!("{} MWh", (sbca.eg_c as u32).separate_with_commas()),
        ],
    ]
}
//use sglab02_lib::sg::prc1::SubstInfo;
//use sglib03::p_31::ld_sb_eb_proj;
//use sglib03::p_31::ld_sb_et_proj;
//use sglib03::p_31::ld_sb_ev_proj;

pub const DOCX0_PATH: &str = "./out/docx0";
pub const DOCX_PATH: &str = "./out/docx";
pub const PDF_PATH: &str = "./out/pdf";
*/
