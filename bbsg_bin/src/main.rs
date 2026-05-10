use std::env;
use std::error::Error;

use bb::asm::ASM::*;
use bb::sty3::AssSumEnum::*;
use bbsg_a10 as bb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let now = std::time::SystemTime::now();
    let a1 = env::args().nth(1).unwrap_or("?".to_string());
    match a1.as_str() {
        "B23" => {
            // Smart Grid Stage 3 summarize transformer into subst, branch
            let coreno = env::args().nth(2).unwrap_or("9".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("SGDT-A11".to_string());
            bb::utl6::sg_proc4(coreno, &vwnm)?;
            bb::sty3::sum_proc_e(coreno, &vwnm)?;
        }
        "B4" => {
            // check evcurv
            let vwnm = env::args().nth(2).unwrap_or("SGDT-A11".to_string());
            bb::utl8::check_ev_curv(&vwnm)?;
        }
        "B3" => {
            // check substation reverse flow with bat
            let vwnm = env::args().nth(2).unwrap_or("SGDT-A11".to_string());
            bb::utl8::sub_rpf_test(&vwnm)?;
        }
        "Z4" => {
            // gen doc1
            let vwnm = env::args().nth(2).unwrap_or("report3".to_string());
            let fnm = env::args().nth(3).unwrap_or("docx-name".to_string());
            bb::utl6::gen_doc1(&vwnm, &fnm)?;
        }
        "B2" => {
            // make substation specific information
            let vwnm = env::args().nth(2).unwrap_or("SGDT-A11".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            bb::stg1::make_sub_ex1(&arif.assumption())?;
        }
        "B1" => {
            // Smart Grid Stage 3 summarize transformer into subst, branch
            let coreno = env::args().nth(2).unwrap_or("9".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("SGDT-A11".to_string());
            bb::sty3::sum_proc_e(coreno, &vwnm)?;
        }
        "Y4" => {
            // new structure with size and inventory
            let vwnm = env::args().nth(2).unwrap_or("SGDT-A11".to_string());
            bb::utl8::sub_excel_rpf_ovl(&vwnm)?;
        }
        "Y3" => {
            // new structure with size and inventory
            let vwnm = env::args().nth(2).unwrap_or("SGDT-A11".to_string());
            bb::utl7::econ_calc_file(&vwnm)?;
        }
        "Y2" => {
            // new structure with size and inventory
            bb::utl7::make_sub_phys()?;
        }
        "Y1" => {
            // new structure with size and inventory
            bb::utl7::read_branch1()?;
        }
        "Z9" => {
            // Smart Grid Stage 3 summarize transformer into subst, branch
            let coreno = env::args().nth(2).unwrap_or("9".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("SGDT-A11".to_string());
            bb::utl6::sg_proc5(coreno, &vwnm)?;
        }
        "Z8" => {
            // Smart Grid stage 2 and save raw transformer assessment
            let coreno = env::args().nth(2).unwrap_or("9".to_string());
            let vwnm = env::args().nth(3).unwrap_or("SGDT-A11".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            bb::utl6::sg_proc4(coreno, &vwnm)?;
        }
        "P072" => {
            // report all PEA branches
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-07".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 7)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumPrvBrn2, 2)?;
        }
        "P08" => {
            // report all PEA branches
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-08".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 8)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumBrn2, 1)?;
        }
        "P07" => {
            // report all PEA branches
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-07".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 7)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumPrvBrn2, 1)?;
        }
        "P06" => {
            // report all PEA branches
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-06".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 6)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumBrn1, 1)?;
        }
        "P05" => {
            // report all PEA branches
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-05".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 5)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumPrvBrn1, 1)?;
        }
        "P04" => {
            // report all PEA branches
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-04".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 4)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumBrn, 1)?;
        }
        "P03" => {
            // report all PEA branches
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-03".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 3)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumPrvBrn, 1)?;
        }
        "P02" => {
            // report all substation
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-02".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 2)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumSub, 1)?;
        }
        "P01" => {
            // report all provinces summarized from substation
            let vwnm = env::args().nth(2).unwrap_or("SGREPO-01".to_string());
            let mut arif = bb::utl6::get_archi_info()?;
            bb::utl6::get_assum_in_view(&vwnm, &mut arif)?;
            //bbsg_a10::utl6::excel_repo1(&arif, 1)?;
            bbsg_a10::utl8::excel_repo1(&arif, SumPrvSub, 1)?;
        }
        "Z3" => {
            // web page
            let vwnm = env::args().nth(2).unwrap_or("SGDT-A11".to_string());
            bbsg_a10::utl6::page(&vwnm).await?;
        }
        /*
        "Z2" => {
            // calculation step 2
            let coreno = env::args().nth(2).unwrap_or("9".to_string());
            let vwnm = env::args().nth(3).unwrap_or("SGDT-A11".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            bb::utl6::sg_proc2(coreno, &vwnm)?;
        }
        */
        "Z1" => {
            // calculation step 1
            let vwnm = env::args().nth(2).unwrap_or("SGDT-A11".to_string());
            bb::utl6::sg_proc1(&vwnm)?;
        }
        "G6" => {
            // check number meters
            bb::utl7::read_aoj2()?;
        }
        "G5" => {
            // check number meters
            bb::utl7::read_aoj1()?;
        }
        "G4" => {
            // check number meters
            bb::utl7::read_meter1()?;
        }
        "G3" => {
            // check number transformer
            bb::utl7::read_trans1()?;
        }
        "G2" => {
            // reading from shapefile part 1
            bb::utl7::read_gis2()?;
        }
        "G1" => {
            // reading from shapefile part 2
            bb::utl7::read_gis1()?;
        }
        "A2" => {
            let arif = bb::utl6::get_archi_info()?;
            if let Some(root) = arif.e_root {
                println!("root OK");
                bb::utl6::archi_out0(root, "out5.archimate")?;
            }
        }
        "C1" => {
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::utl5::chk_aoj1()?;
        }
        /*
        "X6" => {
            let coreno = env::args().nth(2).unwrap_or("5".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::prc2::load(coreno, vwnm)?;
        }
        "X5" => {
            let coreno = env::args().nth(2).unwrap_or("5".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            bbsg_a10::prc2::stage_02(coreno, vwnm)?;
        }
        "X4" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            bbsg_a10::stx3::subass_to_prvass(&ac)?;
        }
        "X3" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            bbsg_a10::stx3::proc(&ac)?;
        }
        "S5" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_prv_2.xlsx".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::utl5::excel_prv_repo2(xlsx.as_str())?;
        }
        "AOJ" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_aoj_1.xlsx".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::utl5::excel_aoj_repo1(xlsx.as_str(), None)?;
        }
        "S4" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_sub_1.xlsx".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::utl5::excel_sub_repo1(xlsx.as_str(), Some(25))?;
        }
        "S3" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_prv_1.xlsx".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::utl5::excel_prv_repo1(xlsx.as_str(), Some(25))?;
        }
        "S2" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_sub_1.xlsx".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::utl5::excel_sub_repo1(xlsx.as_str(), None)?;
        }
        "S1" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let xlsx = env::args().nth(3).unwrap_or("repo_prv_1.xlsx".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::utl5::excel_prv_repo1(xlsx.as_str(), None)?;
        }
        "R1" => {
            let vwnm = env::args().nth(2).unwrap_or("E:report3".to_string());
            let fnm = env::args().nth(3).unwrap_or("doc-repo".to_string());
            bbsg_a10::utl4::repo1(&vwnm, &fnm)?;
        }
        "web1" => {
            //let vwid = env::args().nth(2).unwrap_or("?".to_string());
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            bbsg_a10::p09::web1(vwnm).await?;
        }
        "AR6" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            bbsg_a10::utl4::archi6(&vwnm)?;
        }
        "X2" => {
            let coreno = env::args().nth(2).unwrap_or("5".to_string());
            let coreno = coreno.parse::<usize>().unwrap();
            let vwnm = env::args().nth(3).unwrap_or("E:SGCALC1".to_string());
            bbsg_a10::stx2::stage_02(coreno, vwnm)?;
        }
        "01" => {
            let vwnm = env::args().nth(2).unwrap_or("E:SGCALC1".to_string());
            let ac = bbsg_a10::utl4::make_archi(&vwnm)?;
            let dnm = ac.t(OUTDIR);
            bbsg_a10::dcl::set_dirnm(&dnm);
            bbsg_a10::stg1::stage_01()?;
        }
        "CM3" => bbsg_a10::utl3::check_aoj()?,
        "CM2" => bbsg_a10::utl3::excel_cmd2()?,
        "CM1" => bbsg_a10::utl3::excel_cmd1()?,
        "AR5" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            let fnm = env::args().nth(3).unwrap_or("docx-name".to_string());
            bbsg_a10::utl2::archi5(&vwid, &fnm)?;
        }
        "AR4" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            let fnm = env::args().nth(3).unwrap_or("docx-name".to_string());
            bbsg_a10::utl2::archi4(&vwid, &fnm)?;
        }
        "AR3" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            bbsg_a10::utl2::archi_vids(&vwid)?;
        }
        "AR2" => {
            let vwid = env::args().nth(2).unwrap_or("?".to_string());
            bbsg_a10::utl2::archi2(&vwid)?;
        }
        "AR1" => {
            //bbsg_a10::utl2::archi1()?;
        }
        */
        n => {
            println!("'{}' NG command", n);
        }
    }
    let se = now.elapsed().unwrap().as_secs();
    let mi = se / 60;
    println!("time {se} sec = {mi} min");
    Ok(())
}
