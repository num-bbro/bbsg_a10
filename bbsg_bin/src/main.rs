use std::env;
use std::error::Error;

use bbsg_a09::asm::ASM::OUTDIR;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let now = std::time::SystemTime::now();
    let a1 = env::args().nth(1).unwrap_or("?".to_string());
    match a1.as_str() {
        "C1" => {
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::utl5::chk_aoj1()?;
        }
        "X6" => {
            let coreno = env::args().nth(2).unwrap_or("5".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::prc2::load(coreno, vwnm)?;
        }
        "X5" => {
            let coreno = env::args().nth(2).unwrap_or("5".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            bbsg_a09::prc2::stage_02(coreno, vwnm)?;
        }
        "X4" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            bbsg_a09::stx3::subass_to_prvass(&ac)?;
        }
        "X3" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            bbsg_a09::stx3::proc(&ac)?;
        }
        "S5" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_prv_2.xlsx".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::utl5::excel_prv_repo2(xlsx.as_str())?;
        }
        "AOJ" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_aoj_1.xlsx".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::utl5::excel_aoj_repo1(xlsx.as_str(), None)?;
        }
        "S4" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_sub_1.xlsx".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::utl5::excel_sub_repo1(xlsx.as_str(), Some(25))?;
        }
        "S3" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_prv_1.xlsx".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::utl5::excel_prv_repo1(xlsx.as_str(), Some(25))?;
        }
        "S2" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_sub_1.xlsx".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::utl5::excel_sub_repo1(xlsx.as_str(), None)?;
        }
        "S1" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_prv_1.xlsx".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::utl5::excel_prv_repo1(xlsx.as_str(), None)?;
        }
        "R1" => {
            let vwnm = env::args().nth(2).unwrap_or("E:report3".to_string());
            let fnm = env::args().nth(3).unwrap_or("doc-repo".to_string());
            bbsg_a09::utl4::repo1(&vwnm, &fnm)?;
        }
        "web1" => {
            //let vwid = env::args().nth(2).unwrap_or("?".to_string());
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            bbsg_a09::p09::web1(vwnm).await?;
        }
        "AR6" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            bbsg_a09::utl4::archi6(&vwnm)?;
        }
        "X2" => {
            let coreno = env::args().nth(2).unwrap_or("5".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            bbsg_a09::stx2::stage_02(coreno, vwnm)?;
        }
        "01" => {
            let vwnm = env::args().nth(2).unwrap_or("C:SGCALC1".to_string());
            let ac = bbsg_a09::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a09::dcl::set_dirnm(&dnm);
            bbsg_a09::stg1::stage_01()?;
        }
        "CM3" => bbsg_a09::utl3::check_aoj()?,
        "CM2" => bbsg_a09::utl3::excel_cmd2()?,
        "CM1" => bbsg_a09::utl3::excel_cmd1()?,
        "AR5" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            let fnm = env::args().nth(3).unwrap_or("docx-name".to_string());
            bbsg_a09::utl2::archi5(&vwid, &fnm)?;
        }
        "AR4" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            let fnm = env::args().nth(3).unwrap_or("docx-name".to_string());
            bbsg_a09::utl2::archi4(&vwid, &fnm)?;
        }
        "AR3" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            bbsg_a09::utl2::archi_vids(&vwid)?;
        }
        "AR2" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            bbsg_a09::utl2::archi2(&vwid)?;
        }
        "AR1" => {
            //bbsg_a09::utl2::archi1()?;
        }
        n => {
            println!("'{}' NG command", n);
        }
    }
    let se = now.elapsed().unwrap().as_secs();
    let mi = se / 60;
    println!("time {se} sec = {mi} min");
    Ok(())
}
