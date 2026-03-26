use axum::routing::get;
use std::error::Error;

pub async fn web1(vwid: String) -> Result<(), Box<dyn Error>> {
    let ac = crate::utl4::make_archi(&vwid)?;
    let dnm = ac.t(crate::asm::ASM::OUTDIR);
    println!("dnm:{dnm}");
    crate::dcl::set_dirnm(&dnm);
    println!("web1");
    //let x: axum::routing::MethodRouter = get(crate::web::sbb01::sbb01);
    let app = axum::Router::new()
        .route("/fdw01x", get(crate::web::fdw01x::page))
        .route("/fda01x", get(crate::img::fda01x::get_image_x))
        // sub image
        .route("/fda01", get(crate::img::fda01::get_image))
        .route("/fda02", get(crate::img::fda02::get_image))
        .route("/fda03", get(crate::img::fda03::get_image))
        .route("/fdw01", get(crate::web::fdw01::page))
        .route("/fdw02", get(crate::web::fdw02::page))
        .route("/fdw03", get(crate::web::fdw03::page))
        // sub feeder
        .route("/tra01", get(crate::web::tra01::page))
        // field
        .route("/sbb01", get(crate::web::sbb01::page))
        .route("/sbb02", get(crate::web::sbb02::page))
        .route("/sbb03", get(crate::web::sbb03::page))
        .route("/sbb04", get(crate::web::sbb04::page))
        .route("/sbb05", get(crate::web::sbb05::page))
        .route("/sbb06", get(crate::web::sbb06::page))
        .route("/sbb07", get(crate::web::sbb07::page))
        .route("/sbb08", get(crate::web::sbb08::page))
        .route("/sbb09", get(crate::web::sbb09::page))
        .route("/sbb10", get(crate::web::sbb10::page))
        .route("/sbb11", get(crate::web::sbb11::page))
        .route("/sbb12", get(crate::web::sbb12::page))
        .route("/sbb13", get(crate::web::sbb13::page))
        .route("/sbb14", get(crate::web::sbb14::page))
        .route("/sbb15", get(crate::web::sbb15::page))
        .route("/sbb16", get(crate::web::sbb16::page))
        // sub
        .route("/sba01", get(crate::sba01::sba01))
        .route("/sba02", get(crate::sba02::sba02))
        .route("/sba03", get(crate::sba03::sba03))
        // sub
        .route("/sb01", get(crate::sb01::sb01))
        .route("/sb02", get(crate::sb02::sb02))
        .route("/sb03", get(crate::sb03::sb03))
        .route("/sb04", get(crate::sb04::sb04))
        .route("/sb05", get(crate::sb05::sb05))
        // trans
        .route("/tr01", get(crate::tr01::tr01))
        .route("/tr02", get(crate::tr02::tr02))
        .route("/tr03", get(crate::tr03::tr03))
        .route("/tr04", get(crate::tr04::tr04))
        .route("/tr05", get(crate::tr05::tr05))
        .route("/tr06", get(crate::tr06::tr06))
        // ___
        /*
        .route("/a01", get(crate::a01::a01))
        .route("/a02", get(crate::a02::a02))
        .route("/a03", get(crate::a03::a03))
        .route("/q02", get(crate::web::q02::q02))
        .route("/p02", get(crate::web::p02::p02))
        .route("/p03", get(crate::web::p03::p03))
        .route("/p04", get(crate::web::p04::p04))
        .route("/p05", get(crate::web::p05::p05))
        .route("/p06", get(crate::web::p06::p06))
        .route("/p07", get(crate::web::p07::p07))
        .route("/p08", get(crate::web::p08::p08))
        .route("/m01", get(crate::m01::m01))
        .route("/m02", get(crate::m02::m02))
        */
        .route("/", get(crate::sba01::sba01));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

use crate::utl::load_xlsx;
use regex::Regex;

pub fn sub_load() -> Result<(), Box<dyn Error>> {
    let sbl = "/mnt/e/CHMBACK/pea-data/substation_forecast_2566/Report Substation Load Forecast - Report.xlsx";
    println!("sub station load forecast '{sbl}'");
    let xls = load_xlsx(&vec![sbl])?;
    let tmpt = Regex::new(r".*(KHLONG LUANG)|(PA TONG)|(HUA HIN).+").unwrap();
    for (_s, sht) in xls.iter().enumerate() {
        //println!("{s} {} {} {}", sht.name, sht.shnm, sht.data.len());
        let mut yrs = Vec::<String>::new();
        for rw in sht.data.iter().skip(2).take(1) {
            for cl in rw.iter().skip(2) {
                yrs.push(cl.to_string());
            }
        }
        for rw in sht.data.iter().skip(3) {
            let no = rw[0].to_string();
            let Ok(_no) = no.parse::<i32>() else {
                continue;
            };
            let sb = rw[1].to_string();
            let mut dts = Vec::<f32>::new();
            for cl in rw.iter().skip(2) {
                if let Ok(d) = cl.parse::<f32>()
                    && d > 0.0
                {
                    dts.push(d);
                }
            }
            let dts: Vec<_> = dts.iter().skip(6).collect();
            let dfs = &dts
                .windows(2)
                .map(|w| (w[1] - w[0]) / w[0] * 100.0)
                .collect::<Vec<_>>();
            //let inc = dfs.iter().skip(3).sum::<f32>();
            let inc = dfs.iter().sum::<f32>();
            if inc <= 0.0 {
                continue;
            }
            if !tmpt.is_match(&sb) {
                continue;
            }
            println!("{sb}]");
            println!(" {dfs:?}");
            println!(" {yrs:?}");
        }
    }
    Ok(())
}

pub const SURVEY_SOLAROOF: [(f32, f32, &str); 88] = [
    //Passorn3
    (14.033_022, 100.665_99, "Passorn3"),
    (14.032_772, 100.666_82, "Passorn3"),
    (14.032_328, 100.666_48, "Passorn3"),
    (14.032_572, 100.667_01, "Passorn3"),
    (14.033_222, 100.669_48, "Passorn3"),
    (14.033_286, 100.670_28, "Passorn3"),
    (14.033_081, 100.673_02, "Passorn3"),
    (14.034_058, 100.665_37, "Passorn3"),
    //Baranee
    (14.032_219, 100.674_66, "Baranee"),
    (14.032_247, 100.674_23, "Baranee"),
    (14.031_714, 100.673_21, "Baranee"),
    (14.031_714, 100.673_34, "Baranee"),
    (14.031_689, 100.672_16, "Baranee"),
    (14.031_681, 100.671_36, "Baranee"),
    (14.031_675, 100.669_68, "Baranee"),
    (14.031_76, 100.668_58, "Baranee"),
    (14.032_117, 100.668_52, "Baranee"),
    (14.032_094, 100.668_68, "Baranee"),
    (14.031_914, 100.665_82, "Baranee"),
    (14.031_908, 100.664_92, "Baranee"),
    (14.031_994, 100.670_14, "Baranee"),
    //Passorn 2
    (14.030_647, 100.663_22, "Passorn2"),
    (14.030_428, 100.662_9, "Passorn2"),
    (14.029_692, 100.660_2, "Passorn2"),
    (14.029_497, 100.658_99, "Passorn2"),
    (14.030_236, 100.658_03, "Passorn2"),
    (14.030_253, 100.654_44, "Passorn2"),
    (14.031_119, 100.653_82, "Passorn2"),
    (14.030_986, 100.655_18, "Passorn2"),
    (14.030_881, 100.655_33, "Passorn2"),
    (14.031_147, 100.655_32, "Passorn2"),
    (14.031_372, 100.654_75, "Passorn2"),
    (14.030_683, 100.658_36, "Passorn2"),
    (14.030_669, 100.663_23, "Passorn2"),
    (14.030_436, 100.662_92, "Passorn2"),
    //Passorn 1
    (14.029_548, 100.662_85, "Passorn1"),
    (14.028_731, 100.655_61, "Passorn1"),
    (14.028_772, 100.655_63, "Passorn1"),
    (14.028_958, 100.654_1, "Passorn1"),
    (14.029_193, 100.654_21, "Passorn1"),
    (14.029_16, 100.654_21, "Passorn1"),
    (14.029_564, 100.654_86, "Passorn1"),
    (14.029_08, 100.660_22, "Passorn1"),
    //พฤกษาวิลเลจ 3
    (14.030_034, 100.665_35, "PruksaVill3"),
    (14.029_52, 100.664_31, "PruksaVill3"),
    (14.029_465, 100.671_03, "PruksaVill3"),
    //Big C
    (14.032_6, 100.662_35, "BigC"),
    //ตลาดพาเจริญ
    (14.035_808, 100.662_67, "Pacharoen"),
    //พฤกษา 13
    (14.037_339, 100.662_98, "Pruksa13"),
    (14.037_494, 100.663_01, "Pruksa13"),
    //Bike Park
    (14.037_443, 100.663_31, "BikePark"),
    //CJ MORE
    (14.041_539, 100.663_15, "CJMORE"),
    //ตลาดไทยสมบูรณ์
    (14.045_022, 100.662_82, "ต.ไทยสมบูรณ์"),
    //ม.ไทยสมบูรณ์ 3
    (14.046_167, 100.662_73, "ThaiSombun3"),
    (14.045_411, 100.661_96, "ThaiSombun3"),
    (14.045_581, 100.660_45, "ThaiSombun3"),
    (14.045_558, 100.659_24, "ThaiSombun3"),
    (14.045_939, 100.656_44, "ThaiSombun3"),
    (14.046_358, 100.660_84, "ThaiSombun3"),
    (14.046_783, 100.658_73, "ThaiSombun3"),
    (14.046_642, 100.654_01, "ThaiSombun3"),
    (14.046_464, 100.653_33, "ThaiSombun3"),
    //ม.CHATLUANG16
    (14.047_408, 100.654_28, "ChatLuang16"),
    (14.047_375, 100.655_71, "ChatLuang16"),
    //-- CJ MORE (new)
    (14.048_075, 100.663_13, ""),
    //-- 7/11
    (14.048_972, 100.663_25, ""),
    //--
    (14.051_744, 100.659_77, ""),
    //ม.
    (14.055_369, 100.661_32, ""),
    (14.055_694, 100.660_04, ""),
    (14.055_719, 100.659_3, ""),
    (14.055_464, 100.659, ""),
    //ถ.คลองหลวง แยกคลอง 3
    (14.065_431, 100.661_82, "แยกคลอง๓"),
    (14.064_931, 100.662_15, "แยกคลอง๓"),
    (14.064_781, 100.661_37, "แยกคลอง๓"),
    (14.064_622, 100.660_79, "แยกคลอง๓"),
    (14.066_308, 100.662_08, "แยกคลอง๓"),
    (14.066_25, 100.661_83, "แยกคลอง๓"),
    (14.065_225, 100.666_47, "แยกคลอง๓"),
    //คลองสาม เหนือ
    (14.082_958, 100.663_07, "แยกคลอง๓"),
    (14.083_511, 100.662_92, "แยกคลอง๓"),
    (14.083_769, 100.663_11, "แยกคลอง๓"),
    //ม.
    (14.084_264, 100.662_85, "ม."),
    (14.084_467, 100.661_05, "ม."),
    (14.085_319, 100.660_68, "ม."),
    //
    (14.088_167, 100.662_51, "-"),
    //รง
    (14.103_419, 100.662_06, "รง"),
    (14.102_953, 100.661_55, "รง"),
    (14.106_336, 100.662_89, "รง"),
];

use sglab02_lib::sg::mvline::latlong_utm;
use sglib04::geo1::utm_2_n1d;

pub fn test_1() {
    println!("SET#1");
    for (la, lo, nm) in &SURVEY_SOLAROOF {
        if ["ThaiSombun3", "ChatLuang16"].contains(nm) {
            let utm = latlong_utm(*la, *lo);
            let n1d = utm_2_n1d(utm.0, utm.1);
            println!("{la},{lo} {nm} n1d:{n1d}");
        }
    }
    println!("SET#2");
    for (la, lo, nm) in &SURVEY_SOLAROOF {
        if ["Passorn3", "Baranee"].contains(nm) {
            let utm = latlong_utm(*la, *lo);
            let n1d = utm_2_n1d(utm.0, utm.1);
            println!("{la},{lo} {nm} n1d:{n1d}");
        }
    }
}

// =============================================
pub struct PeaAssDrawMap<'a> {
    pub mg: u32,
    pub updw: u32,
    pub ww: f32,
    pub hh: f32,
    pub ex_x: f32,
    pub ex_y: f32,
    pub or_x: f32,
    pub or_y: f32,
    pub ofs_x: f32,
    pub w: f32,
    pub xx: f32,
    pub yy: f32,
    pub zm: u32,
    pub assv: &'a Vec<PeaAssVar>,
    pub sorf: &'a Vec<u64>,
}

use crate::dcl::Geo;
use crate::dcl::PeaAssVar;
use crate::img::fda01::meter_pixel_to_zoom_lat_2;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::types::Bounds;
use headless_chrome::Browser;
use image::open;
use image::ImageReader;
use image::Rgba;
use image::RgbaImage;
use image_blend::pixelops::pixel_mult;
use image_blend::DynamicChops;
use imageproc::drawing::draw_filled_circle_mut;
use imageproc::drawing::draw_line_segment_mut;
use serde_json::Value;
use sglab02_lib::sg::mvline::utm_latlong;
use sglib04::aoj::zoom_to_meter_pixel_lat;
use sglib04::geo1::find_node;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{thread, time};

pub const MP_WW: f32 = 1800_f32;
pub const MP_HH: f32 = 1733_f32 - 185_f32 * 2.0;
pub const MP_MG: u32 = 72;
pub const MP_UPDW: u32 = 185;

impl<'a> PeaAssDrawMap<'a> {
    //pub fn new(assv: &'a Vec<PeaAssVar>, sub: &'a PeaSub) -> Self {
    pub fn new(assv: &'a Vec<PeaAssVar>, sorf: &'a Vec<u64>) -> Self {
        let mg = MP_MG;
        let updw = MP_UPDW;
        let ww = MP_WW;
        let hh = MP_HH;

        let (w, _h) = (mg as f32 + ww, updw as f32 * 2.0 + hh);

        let fst = assv[0].n1d.n1d_2_utm();
        let (mut x0, mut y0, mut x1, mut y1) = (fst.0, fst.1, fst.0, fst.1);
        for a in assv.iter().take(assv.len() - 1) {
            let pnt = a.n1d.n1d_2_utm();
            x0 = x0.min(pnt.0);
            y0 = y0.min(pnt.1);
            x1 = x1.max(pnt.0);
            y1 = y1.max(pnt.1);
            let (ld, ln) = utm_latlong(pnt.0, pnt.1);
            println!("=== {ld},{ln}");
        }
        let (ox, oy) = ((x1 + x0) * 0.5f32, (y1 + y0) * 0.5f32);
        let wd = x1 - x0;
        let (o_ld, _o_ln) = utm_latlong(ox, oy);
        println!("CENTER {o_ld},{_o_ln}");
        let zm = meter_pixel_to_zoom_lat_2(wd, ww as u32, o_ld);
        let mtpx = zoom_to_meter_pixel_lat(zm, o_ld);

        let ex_x = mtpx * ww;
        let ex_y = mtpx * hh;
        let (o_x, o_y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let (xx, yy) = utm_latlong(o_x, o_y);
        let or_x = o_x - ex_x / 2.0;
        let or_y = o_y - ex_y / 2.0;
        let ofs_x = 40f32;

        PeaAssDrawMap {
            mg,
            updw,
            ww,
            hh,
            ex_x,
            ex_y,
            or_x,
            or_y,
            ofs_x,
            w,
            xx,
            yy,
            zm,
            assv,
            sorf,
        }
    }
}

pub fn load_ggmap(gm: &PeaAssDrawMap, basemap: &str, fpath: &str) {
    let frdrw = format!("{fpath}rw.jpeg");
    loop {
        let frdrw = frdrw.as_str();
        if !std::path::Path::new(frdrw).exists() {
            let bnd = Bounds::Normal {
                left: None,
                top: None,
                width: Some(gm.w.into()),
                height: Some(gm.w.into()),
            };
            let url = "https://www.google.com/maps/@?api=1&map_action=map";
            let url = format!(
                "{url}&center={},{}&zoom={}&basemap={basemap}",
                gm.xx, gm.yy, gm.zm,
            );
            println!("{url}");
            let browser = Browser::default().expect("browser");
            let tab = browser.new_tab().expect("new tab");
            if tab.navigate_to(&url).is_err() {
                println!("!!! fail to navigate to");
                continue;
            }
            if tab.set_bounds(bnd).is_err() {
                println!("!!! fail to set bound");
                continue;
            }
            if tab.wait_until_navigated().is_err() {
                println!("!!! fail to wait");
                continue;
            }

            let ten_millis = time::Duration::from_millis(2000);
            thread::sleep(ten_millis);
            let jpeg_data = tab
                .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)
                .expect("capture");
            std::fs::write(frdrw, jpeg_data).expect("image file");
            println!("img2 = {url} wrote {frdrw}");
        } else {
            println!("image 2 skipped {frdrw}");
        }
        break;
    }
}

pub fn draw_ass_map(map: &PeaAssDrawMap, fpath: &str) {
    let frdrw = format!("{}rw.jpeg", fpath);
    let frd02 = format!("{}02.jpeg", fpath);
    println!("frd02: {frd02}");
    println!("frdrw: {frdrw}");
    if !std::path::Path::new(&frd02).exists()
        && let Ok(img) = ImageReader::open(&frdrw)
        && let Ok(mut img) = img.decode()
    {
        let (w, h) = (img.width(), img.height());
        let img = img.crop(map.mg, map.updw, w - map.mg, h - map.updw * 2);
        let frdcp = format!("{}cp.jpeg", fpath);
        img.save(&frdcp).expect("?");

        let (w, h) = (img.width(), img.height());

        use std::fmt::Write;
        let mut jsmp = String::new();
        writeln!(jsmp, "{{ \"map\": [").expect("?");

        let (t_x, t_y) = (0, 0);
        let fcx = map.ww / map.ex_x;
        let fcy = map.ww / map.ex_x;
        let mut img0 = RgbaImage::new(w, h);
        let mut fg = false;
        for a in map.assv.iter() {
            let pno = format!("หม้อแปลง: {}", a.peano);
            let pnt = a.n1d.n1d_2_utm();
            let (lat, lon) = a.n1d.n1d_2_latlon();
            let x = (pnt.0 - map.or_x) * fcx - map.ofs_x;
            let y = (pnt.1 - map.or_y) * fcy;
            let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
            let (fx, fy) = (ix as f32, iy as f32);
            draw_filled_circle_mut(&mut img0, (ix, iy), 8, Rgba([0, 180, 255, 255]));
            draw_line_segment_mut(
                &mut img0,
                (fx, fy - 10.0),
                (fx, fy + 10.0),
                Rgba([0, 180, 0, 255]),
            );
            draw_line_segment_mut(
                &mut img0,
                (fx - 10.0, fy),
                (fx + 10.0, fy),
                Rgba([0, 180, 0, 255]),
            );
            if fg {
                write!(jsmp, ", ").expect("?");
            }
            writeln!(
                jsmp,
                "{{ \"x\": {ix}, \"y\": {iy}, \"rad\": 10, \"name\": \"{pno}\", \"lat\": {lat}, \"lon\": {lon} }}"
            )
            .expect("?");
            fg = true;
        }

        let (mdx, mdy) = (map.xx, map.yy);

        println!("MID: {mdx},{mdy}");
        println!("t:{t_x},{t_y}  w:{w} h:{h} fac:{fcx},{fcy}");

        //let pnt = map.sub.n1d_f.n1d_2_utm();
        //let x = (pnt.0 - map.or_x) * fcx - map.ofs_x;
        //let y = (pnt.1 - map.or_y) * fcy;
        //let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
        //let (fx, fy) = (ix as f32, iy as f32);
        //draw_filled_circle_mut(&mut img0, (ix, iy), 20, Rgba([255, 150, 255, 255]));

        if fg {
            write!(jsmp, ", ").expect("?");
        }
        let frd01 = format!("{}01.png", fpath);
        img0.save(&frd01).expect("?");

        let mut img1 = open(&frdcp).unwrap();
        let img2 = open(&frd01).unwrap();
        img1.blend(&img2, pixel_mult, true, false).unwrap();
        img1.save(frd02).expect("?");

        writeln!(jsmp, "]}}").expect("?");
        let fmp02 = format!("{}02.json", fpath);
        std::fs::write(fmp02, jsmp).unwrap();
    }
}

use crate::dcl::VarType;
use sglib04::geo1::n1d_2_utm;

pub fn make_img(fdid: &str, mptp: &str, f02: &str, sorf: &Vec<u64>) -> Result<(), Box<dyn Error>> {
    let sbid = &fdid[0..3];
    let p02 = Path::new(&f02);
    let nm0 = (&f02[f02.len() - 9..f02.len() - 7]).to_string();
    println!("f02:{f02} nm0:{nm0}");
    let ppr = p02.parent().unwrap();
    println!("'{f02}' does not exist in {ppr:?}");
    if !ppr.exists() {
        std::fs::create_dir_all(ppr).expect("?");
    }
    let dnm = crate::dcl::get_dirnm();
    let Ok(buf) = std::fs::read(format!("{dnm}/{sbid}-rw4.bin")) else {
        return Err("ERROR".into());
    };
    // ==== read rw3 data
    let Ok((assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("ERROR".into());
    };
    let assv1 = assv0
        .iter()
        .filter(|a| a.fdid == fdid && a.own == "P")
        .cloned()
        .collect::<Vec<_>>();
    let mut trxs = Vec::<u64>::new();
    for a in &assv1 {
        trxs.push(a.n1d);
    }
    trxs.sort();
    println!("TRX: {}", trxs.len());

    let mut tris = Vec::<usize>::new();
    for (i, sr) in sorf.iter().enumerate() {
        let sr1 = find_node(*sr, &trxs);
        let (sx, sy) = n1d_2_utm(*sr);
        let (tx, ty) = n1d_2_utm(sr1);
        let (dx, dy) = ((sx - tx).abs(), (sy - ty).abs());
        if dx > 500.0 || dy > 500.0 {
            continue;
        }
        let tri = trxs.iter().position(|n| *n == sr1).unwrap();
        if !tris.contains(&tri) {
            tris.push(tri);
            println!("i:{i} {dx},{dy} at {tri}");
        }
    }

    let mut assv = Vec::<PeaAssVar>::new();
    for (i, ai) in tris.iter().enumerate() {
        let ass = assv1[*ai].clone();
        let pea = ass.peano.to_string();
        let me1 = ass.v[VarType::NoMet1Ph.tousz()].v;
        let me3 = ass.v[VarType::NoMet3Ph.tousz()].v;
        let tr1 = ass.v[VarType::LvPowSatTr.tousz()].v;
        let tr2 = ass.v[VarType::MvPowSatTr.tousz()].v;
        println!("{i} {pea} {me1} {me3} {tr1} {tr2}");
        assv.push(ass);
    }
    /*
    let Ok(buf) = std::fs::read(format!("{DNM}/{sbid}.bin")) else {
        return Err("ERROR".into());
    };
    let (peasb, _): (PeaSub, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
    */
    let map = PeaAssDrawMap::new(&assv, sorf);
    let ppr0 = ppr.to_str().unwrap();
    //let fpath = format!("{}/{}-rd", ppr0, fdid);
    let fpath = format!("{ppr0}/{fdid}-{nm0}");
    println!("=== 1 : {p02:?} exists : {}", p02.exists());
    load_ggmap(&map, mptp, fpath.as_str());
    draw_ass_map(&map, fpath.as_str());
    println!("=== 2 : {p02:?} exists : {}", p02.exists());
    Ok(())
}

pub fn get_img(
    fdid: &str,
    mptp: &str,
    f02: &str,
    sorf: &Vec<u64>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let p02 = Path::new(&f02);
    if !p02.exists() {
        make_img(fdid, mptp, f02, sorf)?;
    }
    if p02.exists() {
        let mut f = File::open(p02).expect("no file found");
        let metadata = fs::metadata(p02).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("buffer overflow");
        return Ok(buffer);
    }
    Err("ERROR".into())
}

pub fn get_map(
    fdid: &str,
    mptp: &str,
    f02: &str,
    sorf: &Vec<u64>,
) -> Result<Value, Box<dyn Error>> {
    let p02 = Path::new(&f02);
    if !p02.exists() {
        make_img(fdid, mptp, f02, sorf)?;
    }
    if p02.exists() {
        let mp = std::fs::read_to_string(p02).expect("?");
        let v: Value = serde_json::from_str(mp.as_str()).expect("?");
        return Ok(v);
    }
    Err("ERROR".into())
}

pub fn check_sorf() {
    let mut sorf = Vec::<u64>::new();
    for (la, lo, nm) in &SURVEY_SOLAROOF {
        if ["ThaiSombun3", "Baranee"].contains(nm) {
            let utm = latlong_utm(*la, *lo);
            let n1d = utm_2_n1d(utm.0, utm.1);
            sorf.push(n1d);
            //println!("{}. {la},{lo} {nm} n1d:{n1d}", sorf.len());
        }
    }
    let sbid = "KLO";
    //let fdid = "KLO03";
    let dnm = crate::dcl::get_dirnm();
    let Ok(buf) = std::fs::read(format!("{dnm}/{sbid}-rw4.bin")) else {
        println!("ERROR #1");
        return;
    };
    let Ok((assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        println!("ERROR #2");
        return;
    };
    //let assv1 = assv0;
    let assv1 = assv0
        .iter()
        //.filter(|a| a.fdid == fdid && a.own == "P")
        .filter(|a| {
            a.own == "P"
                && (a.v[VarType::NoMet1Ph.tousz()].v + a.v[VarType::NoMet1Ph.tousz()].v) >= 1.0
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut trxs = Vec::<u64>::new();
    for a in &assv1 {
        trxs.push(a.n1d);
    }
    trxs.sort();
    println!("=======>> trx : {}", trxs.len());

    let mut tris = Vec::<usize>::new();
    for sr in sorf.iter() {
        let sr1 = find_node(*sr, &trxs);
        let (sx, sy) = n1d_2_utm(*sr);
        let (tx, ty) = n1d_2_utm(sr1);
        let (dx, dy) = ((sx - tx).abs(), (sy - ty).abs());
        if dx > 500.0 || dy > 500.0 {
            continue;
        }
        let tri = trxs.iter().position(|n| *n == sr1).unwrap();
        if !tris.contains(&tri) {
            tris.push(tri);
            //println!("i:{i} {dx},{dy} at {tri}");
        }
    }

    let mut assv = Vec::<PeaAssVar>::new();
    for (i, ai) in tris.iter().enumerate() {
        let ass = assv1[*ai].clone();
        let pea = ass.peano.to_string();
        let me1 = ass.v[VarType::NoMet1Ph.tousz()].v;
        let me3 = ass.v[VarType::NoMet3Ph.tousz()].v;
        let tr1 = ass.v[VarType::LvPowSatTr.tousz()].v;
        let tr2 = ass.v[VarType::MvPowSatTr.tousz()].v;
        let cap = ass.v[VarType::PwCapTr.tousz()].v;
        let n1d = ass.n1d;
        let sb = ass.sbid.clone();
        println!("{i:02} sb:{sb} n1d:{n1d} {pea} {me1:2} {me3:2} {tr1:.3} {tr2:.3} {cap:6.2}");
        assv.push(ass);
    }
}

pub fn test_2() {
    println!("SET#1");
    let mut sorf = Vec::<u64>::new();
    for (la, lo, nm) in &SURVEY_SOLAROOF {
        //if ["ThaiSombun3", "ChatLuang16"].contains(nm) {
        if ["ThaiSombun3"].contains(nm) {
            let utm = latlong_utm(*la, *lo);
            let n1d = utm_2_n1d(utm.0, utm.1);
            sorf.push(n1d);
            println!("{la},{lo} {nm} n1d:{n1d}");
        }
    }
    let fdid = "KLO03";
    let dnm = crate::dcl::get_dirnm();
    let f02 = format!("{dnm}/fdimg2/{fdid}-rd02.jpeg");
    let _ = get_img(fdid, "roadmap", &f02, &sorf);
}

pub fn test_3() {
    println!("SET#1");
    let mut sorf = Vec::<u64>::new();
    for (la, lo, nm) in &SURVEY_SOLAROOF {
        //if ["Passorn3", "Baranee"].contains(nm) {
        if ["Baranee"].contains(nm) {
            let utm = latlong_utm(*la, *lo);
            let n1d = utm_2_n1d(utm.0, utm.1);
            sorf.push(n1d);
            println!("{la},{lo} {nm} n1d:{n1d}");
        }
    }
    let fdid = "KLO03";
    let dnm = crate::dcl::get_dirnm();
    let f02 = format!("{dnm}/fdimg3/{fdid}-rd02.jpeg");
    let _ = get_img(fdid, "roadmap", &f02, &sorf);
}
