
use crate::asm::ASM::*;
use crate::dcl::*;
use crate::utl::*;
use std::collections::HashMap;
use std::error::Error;
use crate::sty2::ass_rank;
use std::sync::Arc;
use std::sync::Mutex;
use crate::utl7::get_brn_map;
use crate::sty2::SUB_LEVEL_FLDS;
use crate::sty2::ass_calc;
use crate::sty2::PRV_LEVEL_FLDS;
use serde_json::Value;
use crate::utl4::NumValEnum;
use crate::utl6::archi_extract0;
use crate::utl6::ARCHI_INPUT;
use crate::utl6::archi_xml_read0;
use crate::utl6::archi_analyze;
use crate::utl6::get_assum_in_view;
use crate::utl6::Assumption;
use strum_macros::EnumString;
use strum_macros::EnumIter;
use bincode::Encode;
use bincode::Decode;
use crate::ben3::ben_bess_calc;

#[derive(Debug, Clone, Encode, Decode, EnumIter, EnumString)]
pub enum AssSumEnum {
    SumSub, // 2
    SumBrn, // 4
    SumBrn1, // 6
    SumBrn2, // 8
    SumPrvSub, // 1
    SumPrvBrn, // 3
    SumPrvBrn1, // 5
    SumPrvBrn2, // 7
}

pub fn sum_proc_e(coreno: usize, vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    let fnm = format!("{dnm}/all-rw4.bin");
    println!("..sg_proc5 fnm:{fnm}");
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("NO {fnm} file:").into());
    };
    println!("READ BUF {}", buf.len());
    let Ok((mut assv, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Failed to decode rw3:".into());
    };
    println!("assv: {}", assv.len());
    sum_stage_e(coreno, &arif.assumption(), &mut assv)?;
    Ok(())
}


pub fn sum_stage_e(
    coreno: usize,
    ass: &Assumption,
    tras_raw: &mut [PeaAssVar],
) -> Result<(), Box<dyn Error>> {

    let dnm = ass.t(OUTDIR);
    let mut subass = sum_stage_sub(coreno, ass, tras_raw)?;

    //==== populate back to transformer
    {
        let pop_flds = [VarType::MaxPosPowSub, VarType::FirMvReThb, VarType::FirBatSubSave, VarType::FirBatSvgSave, VarType::FirBatPriceDiff, VarType::FirBatEnerSave];

        let tik = std::time::SystemTime::now();
        for (subass, trasis) in subass.iter() {
            let sid = subass.sbid.clone();
            let mut sbrw = Vec::<PeaAssVar>::new();
            for ti in trasis.iter() {
                let rat = tras_raw[*ti].v[VarType::AllSellTr.tousz()].v
                    / z2o(subass.v[VarType::AllSellTr.tousz()].v);
                for pf in pop_flds.iter() {
                    tras_raw[*ti].vy[pf.tousz()] = subass.vy[pf.tousz()]
                        .iter()
                        .map(|v| *v * rat)
                        .collect::<Vec<_>>();
                    tras_raw[*ti].yr_sum(pf.clone(), ass);
                }
                sbrw.push(tras_raw[*ti].clone());
            }
            let fnm = format!("{dnm}/{sid}-rw4.bin");
            //println!("save {sid} - v:{} -> {fnm}", trasis.len());
            let bin: Vec<u8> = bincode::encode_to_vec(&sbrw, bincode::config::standard()).unwrap();
            std::fs::write(fnm, bin).unwrap();
        }
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE FILES:{} - {se}sec", subass.len());
    }


    let mut brn_ass = sum_stage_brn(coreno, ass, tras_raw)?;

    let mut prv_sub_ass = sum_stage_prv_sub(coreno, ass, &mut subass)?;

    let (prv_brn_ass, pv_brns) = sum_stage_prv_brn(ass, &mut prv_sub_ass, &mut brn_ass)?;
    println!("BRANCH #0 {} {}", prv_brn_ass.len(), pv_brns.len());

    //let (prv_brn_ass,pv_brns) = sum_stage_prv_brn(coreno, ass, &mut prv_sub_ass, &mut brn_ass)?;
    //let (prv_brn_ass,pv_brns) = sum_stage_prv_brn(coreno, ass, &mut prv_sub_ass, &mut brn_ass)?;

    let (mut brn_ass1, brn_mb1) = sum_stage_brn1_b(coreno, ass, &mut brn_ass)?;
    //let (mut brn_ass1, brn_mb1) = sum_stage_brn1(coreno, ass, &mut brn_ass)?;
    println!("PROV BRN {} - {}", brn_ass1.len(), brn_mb1.len());

    //let (prv_brn_ass1, pv_brns1) = sum_stage_prv_brn1(ass, &mut prv_sub_ass, &mut brn_ass1, &brn_mb1)?;
    //let prv_brn_ass1 = sum0_stage_prv_brn1(ass, &mut prv_sub_ass, &mut brn_ass1)?;
    let (prv_brn_ass1, pv_brns1) = sum_stage_prv_brn2(ass, &mut prv_sub_ass, &mut brn_ass1)?;
    //println!("BRANCH #1 {} {}", prv_brn_ass1.len(), pv_brns1.len());

    let fcrate = ass.v(BRN_MIN_FIR_CST_RATIO);
    let mut brn_ass2 = Vec::<(PeaAssVar,Vec<usize>)>::new();
    //let mut brn_mb2 = HashMap::<usize,Vec<usize>>::new();
    for (_i,assi) in brn_ass1.iter().enumerate() {
        let fcr = assi.0.v[VarType::FirCstRate as usize].v;
        if fcr<fcrate { continue; }
        /*
        let Some(mis) = brn_mb1.get(&i) else {
            println!("ERROR #4 {}", assi.0.aojcd);
            continue;
        };
        */
        //println!("{i}.{} FCR:{fcr}", assi.0.aojcd);
        let ass2 = assi.clone();
        brn_ass2.push(ass2);
        //brn_mb2.insert(i, mis.clone());
    }

    //let (prv_brn_ass2, pv_brns2) = sum_stage_prv_brn(ass, &mut prv_sub_ass, &mut brn_ass2)?;
    //let prv_brn_ass2 = sum_stage_prv_brn1(ass, &mut prv_sub_ass, &mut brn_ass2)?;
    let (prv_brn_ass2, pv_brns2) = sum_stage_prv_brn2(ass, &mut prv_sub_ass, &mut brn_ass2)?;
    //println!("BRANCH #2 {} {}", prv_brn_ass2.len(), pv_brns2.len());

    let mut brn2_hs = HashSet::<String>::new();
    for (ass,_) in brn_ass2.iter() {
        brn2_hs.insert(ass.aojcd.clone());
    }
    let mut brn_ass3 = Vec::<(PeaAssVar,Vec<usize>)>::new();
    for (ass,mb) in brn_ass1.iter_mut() {
        if let Some(_x) = brn2_hs.get(&ass.aojcd) {
            ass.v[VarType::TakeNote as usize].v = 1.0;
        } else {
            ass.v[VarType::TakeNote as usize].v = 1.0;
        }
        brn_ass3.push((ass.clone(), mb.clone()));
    }

    //==== SAVE SUMMARY FILE
    {
        let tik = std::time::SystemTime::now();

        let bin: Vec<u8> = bincode::encode_to_vec(&subass, bincode::config::standard()).unwrap();
        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumSub);
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumPrvSub);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv_sub_ass, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumBrn);
        let bin: Vec<u8> = bincode::encode_to_vec(&brn_ass, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumPrvBrn);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv_brn_ass, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        /*
        let fnm = format!("{dnm}/asssum-{:?}-m.bin", AssSumEnum::SumPrvBrn);
        let bin: Vec<u8> = bincode::encode_to_vec(&pv_brns, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");
        */

        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumBrn1);
        let bin: Vec<u8> = bincode::encode_to_vec(&brn_ass1, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumPrvBrn1);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv_brn_ass1, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}-m.bin", AssSumEnum::SumPrvBrn1);
        let bin: Vec<u8> = bincode::encode_to_vec(&brn_mb1, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumBrn2);
        let bin: Vec<u8> = bincode::encode_to_vec(&brn_ass2, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}.bin", AssSumEnum::SumPrvBrn2);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv_brn_ass2, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}-b.bin", AssSumEnum::SumPrvBrn1);
        let bin: Vec<u8> = bincode::encode_to_vec(&pv_brns1, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        let fnm = format!("{dnm}/asssum-{:?}-b.bin", AssSumEnum::SumPrvBrn2);
        let bin: Vec<u8> = bincode::encode_to_vec(&pv_brns2, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");

        /*
        let fnm = format!("{dnm}/asssum-{:?}-m.bin", AssSumEnum::SumPrvBrn2);
        let bin: Vec<u8> = bincode::encode_to_vec(&pv_brns2, bincode::config::standard()).unwrap();
        std::fs::write(&fnm, bin).unwrap();
        println!("file '{fnm}' saved");
        */

        //====== SAVE SUB SUMMARY
        let mut subass0 = subass
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut subass0);
        let bin: Vec<u8> = bincode::encode_to_vec(&subass0, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-sbrw.bin"), bin).unwrap();

        //====== SAVE PROV_SUB SUMMARY
        let mut prvass = prv_sub_ass
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut prvass);
        let bin: Vec<u8> = bincode::encode_to_vec(&prvass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-pvrw.bin"), bin).unwrap();

        //====== SAVE BRANCH SUMMARY
        let mut brn_ass = brn_ass
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut brn_ass);
        let bin: Vec<u8> =
            bincode::encode_to_vec(&brn_ass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-branch.bin"), bin).unwrap();

        //====== SAVE PROV_BRANCH SUMMARY
        let mut prv2ass = prv_brn_ass
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut prv2ass);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv2ass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-pvbrn.bin"), bin).unwrap();

        //====== SAVE BRANCH SUMMARY
        let mut brn_ass1 = brn_ass1
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut brn_ass1);
        let bin: Vec<u8> =
            bincode::encode_to_vec(&brn_ass1, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-branch1.bin"), bin).unwrap();

        //====== SAVE PROV_BRANCH2 SUMMARY
        let mut prv_brn_ass1 = prv_brn_ass1
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut prv_brn_ass1);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv_brn_ass1, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-pvbrn1.bin"), bin).unwrap();

        //====== SAVE BRANCH SUMMARY
        let mut brn_ass2 = brn_ass2
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut brn_ass2);
        let bin: Vec<u8> =
            bincode::encode_to_vec(&brn_ass2, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-branch2.bin"), bin).unwrap();

        //====== SAVE PROV_BRANCH2 SUMMARY
        let mut prv_brn_ass2 = prv_brn_ass2
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut prv_brn_ass2);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv_brn_ass2, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-pvbrn2.bin"), bin).unwrap();

        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE summary - {se} sec",);
    }

    Ok(())
}
use crate::utl8::get_sub_rpf_ovl;
use crate::utl8::SubRpfOvl;

pub fn sum_stage_sub(
    coreno: usize,
    ass: &Assumption,
    tras_raw: &mut [PeaAssVar],
) -> Result<Vec<(PeaAssVar,Vec<usize>)>, Box<dyn Error>> {
    let mut subass: Vec<(PeaAssVar,Vec<usize>)>;
    let dnm = ass.t(OUTDIR);
    let me_x = ass.v(METER_NO_MULTIPLY);
    let resc = re_scurv();
    //============  SUMMARY TO SUBST BEGIN ===========
    //============  SUMMARY TO SUBST BEGIN ===========
    //============  SUMMARY TO SUBST BEGIN ===========
    let rpfovl = get_sub_rpf_ovl()?;
    let mut sb_rpfovl = HashMap::<String,SubRpfOvl>::new();
    for ro in rpfovl.iter() {
        sb_rpfovl.entry(ro.sbid.clone()).insert_entry(ro.clone());
    }
    println!("=========== RPF OVL: {}", sb_rpfovl.len());
    let sub_bess: Vec<String> = if let Ok(prvs) = ass.ve(SUB_LIST_ADD_BESS) {
        let mut provs = vec![];
        if let NumValEnum::Json(Value::Array(prvs)) = prvs {
            for x in prvs.iter() {
                let Value::String(x) = x else {
                    continue;
                };
                let s = x.to_string();
                //println!(" {s}");
                provs.push(s);
            }
        }
        provs
    } else {
        vec![]
    };
    println!("BESS SUB LEN: {}", sub_bess.len());
    let a_sub_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar, Vec<usize>)>::new()));
    {
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            let rsz = tras_raw.len().div_ceil(coreno);
            for vsa in tras_raw.chunks(rsz) {
                let c_sub_hm = a_sub_hm.clone();
                s.spawn(move || {
                    let mut sub_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                    for sa in vsa {
                        let sbid = sa.sbid.to_string();
                        let (sbas, sbis) = sub_m
                            .entry(sbid)
                            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                        sbas.add_ex(sa, &SUB_LEVEL_FLDS);
                        sbas.pvid = sa.pvid.clone();
                        sbas.sbid = sa.sbid.clone();
                        sbis.push(sa.ix);
                    }
                    if let Ok(mut sub_hm) = c_sub_hm.lock() {
                        for (sub, mut sbis1) in sub_m.into_values() {
                            let sbid = sub.sbid.to_string();
                            let (sbas, sbis2) = sub_hm
                                .entry(sbid)
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            sbas.add_ex(&sub, &SUB_LEVEL_FLDS);
                            sbas.sbid = sub.sbid.clone();
                            sbas.pvid = sub.pvid.clone();
                            sbis2.append(&mut sbis1);
                        }
                        //println!("  sub chunk {}", sub_m.len());
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== TR => SUBSTATION - {se} secs");
    }
    let sub_hm = a_sub_hm.lock().unwrap().clone();
    drop(a_sub_hm);
    subass = sub_hm.into_values().collect();
    //let mut subass: Vec<_> = sub_hm.into_values().collect();


    let fnm = format!("{dnm}/pea_sub_ex1.bin");
    let buf = std::fs::read(&fnm)?;
    let Ok((sbex1, _)): Result<(HashMap<String,PeaSubEx1>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()) else {
        return Err("Failed to decode rw3:".into());
    };
    println!("PEA_SUB_EX1 >>>>>>>>>>>>>>> {}", sbex1.len());


    //==== SUBSTATION CALCULATE
    {
        let ass = ass.clone();
        let tik = std::time::SystemTime::now();
        let ssz = subass.len().div_ceil(coreno);

        //========== CHECK
        std::thread::scope(|s| {
            let ass = ass.clone();
            for subasc in subass.chunks_mut(ssz) {
                let ass = ass.clone();
                let resc = resc.clone();
                //let ac = ac.clone();
                let sbex1 = sbex1.clone();
                let sb_rpfovl = sb_rpfovl.clone();
                let sub_bess = sub_bess.clone();
                s.spawn(move || {
                    for (vas, _vis) in subasc.iter_mut() {
                        vas.v[VarType::NoMet1PhA.tousz()].v =
                            (vas.v[VarType::NoMet1Ph.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhA.tousz()].v =
                            (vas.v[VarType::NoMet3Ph.tousz()].v * me_x).floor();

                        vas.v[VarType::NoMet1PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet1PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet3PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet1PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet3PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet1PhWis.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet3PhWis.tousz()].v * me_x).floor();

                        vas.v[VarType::ChgStnCap as usize].v =
                            vas.v[VarType::ChgStnCapTr as usize].v;
                        vas.v[VarType::ChgStnSell as usize].v =
                            vas.v[VarType::ChgStnSellTr as usize].v;
                        vas.v[VarType::MvPowSatTr as usize].v =
                            vas.v[VarType::MaxPosPowSub as usize].v
                                / z2o(vas.v[VarType::SubPowCap as usize].v);
                        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                        vas.v[VarType::SmallSell as usize].v =
                            vas.v[VarType::SmallSellTr as usize].v;
                        vas.v[VarType::LargeSell as usize].v =
                            vas.v[VarType::LargeSellTr as usize].v;
                        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                        let v = vas.v[VarType::UnbalPowTr as usize].v
                            / z2o(vas.v[VarType::PwCapTr as usize].v);
                        vas.v[VarType::CntUnbalPow as usize].v =
                            if v > 0.5f32 { 1f32 } else { 0f32 };

                        let pwmx = vas.v[VarType::MaxPosPowSub.tousz()].v;
                        for rt in resc.iter() {
                            let rerev = rt
                                * pwmx
                                * ass.v(PEAK_POWER_RATIO)
                                * ass.v(RENEW_HOUR_PER_DAY)
                                * 365.0
                                * ass.v(RENEW_SAVE_PER_MWH);
                            vas.vy[VarType::FirMvReThb.tousz()].push(rerev);
                        }
                        vas.yr_sum(VarType::FirMvReThb, &ass);
                        // CHECK MEMBERS

                        if let Some(sx) = sbex1.get(&vas.sbid) {
                            vas.n1d = sx.n1d_f;
                            if sub_bess.contains(&vas.sbid) {
                                let (b1,b2,b3,b4,bess) = ben_bess_calc(sx, vas, &ass, &sb_rpfovl);
                                vas.vy[VarType::FirBatSubSave as usize] = b1;
                                vas.vy[VarType::FirBatSvgSave as usize] = b2;
                                vas.vy[VarType::FirBatPriceDiff as usize] = b3;
                                vas.vy[VarType::FirBatEnerSave as usize] = b4;
                                vas.yr_sum(VarType::FirBatSubSave, &ass);
                                vas.yr_sum(VarType::FirBatSvgSave, &ass);
                                vas.yr_sum(VarType::FirBatPriceDiff, &ass);
                                vas.yr_sum(VarType::FirBatEnerSave, &ass);
                                vas.v[VarType::BessMWh as usize].v = bess;
                            }
                                /*
                            if vas.v[VarType::SolarEnergy as usize].v>0.0 {

                                let (b1,b2,b3,b4,bess) = ben_bess_calc(sx, vas, &ass, &sb_rpfovl);
                                vas.vy[VarType::FirBatSubSave as usize] = b1;
                                vas.vy[VarType::FirBatSvgSave as usize] = b2;
                                vas.vy[VarType::FirBatPriceDiff as usize] = b3;
                                vas.vy[VarType::FirBatEnerSave as usize] = b4;
                                vas.yr_sum(VarType::FirBatSubSave, &ass);
                                vas.yr_sum(VarType::FirBatSvgSave, &ass);
                                vas.yr_sum(VarType::FirBatPriceDiff, &ass);
                                vas.yr_sum(VarType::FirBatEnerSave, &ass);
                                vas.v[VarType::BessMWh as usize].v = bess;

                            }
                                */
                        }
                        let _ = ass_calc(vas, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("SUM SUB:{} - {se}sec", subass.len());
    }
    //============  SUMMARY TO SUBST END ===========
    //============  SUMMARY TO SUBST END ===========
    //============  SUMMARY TO SUBST END ===========
    Ok(subass)
}

pub fn sum_stage_brn(
    coreno: usize,
    ass: &Assumption,
    tras_raw: &mut [PeaAssVar],
) -> Result<Vec<(PeaAssVar,Vec<usize>)>, Box<dyn Error>> {

    let mut branch_ass: Vec<(PeaAssVar,Vec<usize>)>;
    let dnm = ass.t(OUTDIR);
    let me_x = ass.v(METER_NO_MULTIPLY);
    let resc = re_scurv();


    //============  SUMMARY TO BRANCH BEGIN ===========
    //============  SUMMARY TO BRANCH BEGIN ===========
    //============  SUMMARY TO BRANCH BEGIN ===========
    let (brns, cd_bri) = get_brn_map()?;
    let a_branch_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar, Vec<usize>)>::new()));
    {
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            //let brns = brns.clone();
            //let cd_bri = cd_bri.clone();
            let rsz = tras_raw.len().div_ceil(coreno);
            for vsa in tras_raw.chunks(rsz) {
                let brns = brns.clone();
                let cd_bri = cd_bri.clone();
                let c_branch_hm = a_branch_hm.clone();
                s.spawn(move || {
                    let mut brn_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                    for sa in vsa {
                        let aojcd = sa.aojcd.to_string();
                        let (bras, bris) = brn_m
                            .entry(aojcd.clone())
                            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                        bras.add_ex(sa, &SUB_LEVEL_FLDS);
                        bras.aojcd = aojcd.clone();
                        if let Some(ii) = cd_bri.get(&aojcd) {
                            if let Some(ji) = &brns[*ii].i_prov {
                                bras.pvid = brns[*ji].name.clone();
                            } else {
                                //println!(">>>>1111  NO parent PROVINCE found {aojcd} : name: {}", brns[*ii].name);
                            }
                            bris.push(sa.ix);
                        } else {
                            //println!(">>>>2222  NO AOJCD has found : {aojcd}");
                        }
                        //bris.push(sa.ix);
                    }
                    if let Ok(mut brn_hm) = c_branch_hm.lock() {
                        for (brn, mut brnis) in brn_m.into_values() {
                            let aojcd = brn.aojcd.clone();
                            let (bras, bris) = brn_hm
                                .entry(aojcd.clone())
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            bras.add_ex(&brn, &SUB_LEVEL_FLDS);
                            bras.aojcd = brn.aojcd.clone();
                            if let Some(ii) = cd_bri.get(&aojcd) && let Some(ji) = &brns[*ii].i_prov {
                                bras.pvid = brns[*ji].name.clone();
                            }
                            let mut bris = bris.clone();
                            brnis.append(&mut bris);
                        }
                        //println!("  sub chunk {}", sub_m.len());
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== TR => BRANCH - {se} secs");
    }
    let branch_hm = a_branch_hm.lock().unwrap().clone();
    drop(a_branch_hm);
    //let mut branch_ass: Vec<_> = branch_hm.into_values().collect();
    branch_ass = branch_hm.into_values().collect();

    //==== CALCULATE
    {
        let ass = ass.clone();
        let tik = std::time::SystemTime::now();
        let ssz = branch_ass.len().div_ceil(coreno);

        //========== CHECK
        std::thread::scope(|s| {
            let ass = ass.clone();
            for subasc in branch_ass.chunks_mut(ssz) {
                let ass = ass.clone();
                let resc = resc.clone();
                //let ac = ac.clone();
                s.spawn(move || {
                    for (vas, _vis) in subasc.iter_mut() {
                        vas.v[VarType::NoMet1PhA.tousz()].v =
                            (vas.v[VarType::NoMet1Ph.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhA.tousz()].v =
                            (vas.v[VarType::NoMet3Ph.tousz()].v * me_x).floor();

                        vas.v[VarType::NoMet1PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet1PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet3PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet1PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet3PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet1PhWis.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet3PhWis.tousz()].v * me_x).floor();

                        vas.v[VarType::ChgStnCap as usize].v =
                            vas.v[VarType::ChgStnCapTr as usize].v;
                        vas.v[VarType::ChgStnSell as usize].v =
                            vas.v[VarType::ChgStnSellTr as usize].v;
                        vas.v[VarType::MvPowSatTr as usize].v =
                            vas.v[VarType::MaxPosPowSub as usize].v
                                / z2o(vas.v[VarType::SubPowCap as usize].v);
                        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                        vas.v[VarType::SmallSell as usize].v =
                            vas.v[VarType::SmallSellTr as usize].v;
                        vas.v[VarType::LargeSell as usize].v =
                            vas.v[VarType::LargeSellTr as usize].v;
                        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                        let v = vas.v[VarType::UnbalPowTr as usize].v
                            / z2o(vas.v[VarType::PwCapTr as usize].v);
                        vas.v[VarType::CntUnbalPow as usize].v =
                            if v > 0.5f32 { 1f32 } else { 0f32 };

                        let pwmx = vas.v[VarType::MaxPosPowSub.tousz()].v;
                        for rt in resc.iter() {
                            let rerev = rt
                                * pwmx
                                * ass.v(PEAK_POWER_RATIO)
                                * ass.v(RENEW_HOUR_PER_DAY)
                                * 365.0
                                * ass.v(RENEW_SAVE_PER_MWH);
                            vas.vy[VarType::FirMvReThb.tousz()].push(rerev);
                        }
                        vas.yr_sum(VarType::FirMvReThb, &ass);
                        // CHECK MEMBERS

                        let _ = ass_calc(vas, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("SUM SUB:{} - {se}sec", branch_ass.len());
    }
    {
        let tik = std::time::SystemTime::now();
        for (subass, trasis) in branch_ass.iter() {
            let aojcd = subass.aojcd.clone();
            let mut sbrw = Vec::<PeaAssVar>::new();
            for ti in trasis.iter() {
                sbrw.push(tras_raw[*ti].clone());
            }
            let fnm = format!("{dnm}/{aojcd}-rw4.bin");
            //println!("save {sid} - v:{} -> {fnm}", trasis.len());
            let bin: Vec<u8> = bincode::encode_to_vec(&sbrw, bincode::config::standard()).unwrap();
            std::fs::write(fnm, bin).unwrap();
        }
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE FILES:{} - {se}sec", branch_ass.len());
    }
    //============  SUMMARY TO BRANCH END ===========
    //============  SUMMARY TO BRANCH END ===========
    //============  SUMMARY TO BRANCH END ===========
    Ok(branch_ass)
}

pub fn sum_stage_prv_sub(
    coreno: usize,
    ass: &Assumption,
    //_tras_raw: &mut [PeaAssVar],
    subass: &mut Vec<(PeaAssVar,Vec<usize>)>,
) -> Result<Vec<(PeaAssVar,Vec<usize>)>, Box<dyn Error>> {

    let mut prvass: Vec<(PeaAssVar,Vec<usize>)>;

    //============= PROVINCE CALCULATION BEGIN ===============
    //============= PROVINCE CALCULATION BEGIN ===============
    //============= PROVINCE CALCULATION BEGIN ===============
    let a_prv_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar,Vec<usize>)>::new()));
    let tik = std::time::SystemTime::now();
    {
        std::thread::scope(|s| {
            let psz = subass.len().div_ceil(coreno);
            for vsa in subass.chunks_mut(psz) {
                let c_prv_hm = a_prv_hm.clone();
                s.spawn(move || {
                    let mut prv_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                    for (sa, si) in vsa {
                        let pvid = sa.pvid.to_string();
                        let (pvas, pvi) = prv_m
                            .entry(pvid.clone())
                            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                        pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        pvas.pvid = pvid.clone();
                        let mut mi = si.clone();
                        pvi.append(&mut mi);
                    }
                    if let Ok(mut prv_hm) = c_prv_hm.lock() {
                        for (_k,(prv, pvi)) in prv_m.iter_mut() {
                            let pvid = prv.pvid.to_string();
                            let (pvas, pvis) = prv_hm
                                .entry(pvid.clone())
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            pvas.pvid = prv.pvid.clone();
                            let mut mi = pvis.clone();
                            pvi.append(&mut mi);
                        }
                    }
                });
            }
        });
    }
    let prv_hm = a_prv_hm.lock().unwrap().clone();
    drop(a_prv_hm);
    //let mut prvass: Vec<_> = prv_hm.into_values().collect();
    prvass = prv_hm.into_values().collect();

    for (vas,_) in prvass.iter_mut() {
        vas.v[VarType::ChgStnCap as usize].v = vas.v[VarType::ChgStnCapTr as usize].v;
        vas.v[VarType::ChgStnSell as usize].v = vas.v[VarType::ChgStnSellTr as usize].v;
        vas.v[VarType::MvPowSatTr as usize].v =
            vas.v[VarType::MaxPosPowSub as usize].v / z2o(vas.v[VarType::SubPowCap as usize].v);
        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
        vas.v[VarType::SmallSell as usize].v = vas.v[VarType::SmallSellTr as usize].v;
        vas.v[VarType::LargeSell as usize].v = vas.v[VarType::LargeSellTr as usize].v;
        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
        vas.v[VarType::TpaZone.tousz()].v /= z2o(vas.v[VarType::NoTr.tousz()].v);
        //let v = vas.v[VarType::UnbalPowTr as usize].v / z2o(vas.v[VarType::PwCapTr as usize].v);
        //vas.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
        let _ = ass_calc(vas, ass);
    }
    let mut pvas_mx = PeaAssVar::from(0u64);
    for (pvas,_) in prvass.iter() {
        pvas_mx.max(pvas);
    }
    let mut pvas_no = prvass.clone();
    for (pvas,_) in pvas_no.iter_mut() {
        pvas.nor(&pvas_mx);
    }
    for ((pvas,_), (pvno,_)) in prvass.iter_mut().zip(pvas_no.iter()) {
        pvas.v[VarType::TpaZone.tousz()].v = pvno.v[VarType::TpaZone.tousz()].v;
        pvas.v[VarType::MaxPosPowSub.tousz()].v = pvno.v[VarType::MaxPosPowSub.tousz()].v;
    }

    let mut we_tpa = PeaAssVar::from(0u64);
    for (vt, vv) in WE_TPA {
        we_tpa.v[vt.tousz()].v = vv;
    }
    let mut tpa_ad = PeaAssVar::from(0u64);
    let mut pvtpa = prvass.clone();
    let _flds = [VarType::TpaFcst.tousz()];
    for ((tpa,_), (prv,_)) in pvtpa.iter_mut().zip(prvass.iter_mut()) {
        tpa.weigh(&we_tpa);
        tpa.sum();
        prv.v[VarType::TpaFcst as usize].v = tpa.res;
        tpa_ad.v[VarType::TpaFcst.tousz()].v += prv.v[VarType::TpaFcst.tousz()].v;
        //tpa_ad.add_ex(prv, &flds);
    }

    let mut pvas_no = prvass.clone();
    for ((no,_), (prv,_)) in pvas_no.iter_mut().zip(prvass.iter_mut()) {
        no.nor(&tpa_ad);
        prv.v[VarType::TpaFcst.tousz()].v = no.v[VarType::TpaFcst.tousz()].v;
    }

    let tpa_day_hours = ass.v(TPA_DAY_HOURS);
    let tpa_price_thb = ass.v(TPA_PRICE_THB);
    let tpa_ben_claim = ass.v(TPA_BEN_CLAIM);
    let tpa_year_days = ass.v(TPA_YEAR_DAYS);
    let tpafcs = ass.ve(TPA_FORECAST)?;
    let NumValEnum::Json(tpa_json) = tpafcs else {
        return Err("No TPA forecast data".into());
    };
    let Value::Array(tpa_ary) = tpa_json else {
        return Err("TPA value is not array".into());
    };
    let mut tpa_fa = vec![];
    for va in tpa_ary {
        let Value::Number(n) = va else {
            continue;
        };
        let Some(f) = n.as_f64() else {
            continue;
        };
        tpa_fa.push(f as f32);
    }

    for (pvas,_) in prvass.iter_mut() {
        let rat = pvas.v[VarType::TpaFcst.tousz()].v;
        let mut tpav = vec![];
        for y in tpa_fa.iter().take(15) {
            let y = y * tpa_day_hours * tpa_price_thb * tpa_year_days;
            let y = y * rat;
            let y = y * tpa_ben_claim;
            let y = y * 1_000f32;
            tpav.push(y);
        }
        pvas.vy[VarType::FirTpaThb.tousz()].append(&mut tpav);
        pvas.yr_sum(VarType::FirTpaThb, ass);
    }

    let se = tik.elapsed().unwrap().as_secs();
    println!("PRV:{} - {se}sec", prvass.len());
    //============= PROVINCE CALCULATION END ===============
  //============= PROVINCE CALCULATION END ===============
    //============= PROVINCE CALCULATION END ===============
    Ok(prvass)
}

pub fn sum_stage_prv_brn1(
    //coreno: usize,
    ass: &Assumption,
    prv_sub_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
    branch_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
    //brn_mbs: &HashMap<usize,Vec<usize>>,
) -> Result<Vec<(PeaAssVar,Vec<usize>)>, Box<dyn Error>> {

    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //let mut pv_brns = HashMap::<String,Vec<String>>::new();
    let mut prv3_hm = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
    for (_ii,(sa,si)) in branch_ass.iter().enumerate() {
        let pvid = sa.pvid.to_string();
        let (pvas, pvi) = prv3_hm
            .entry(pvid.clone())
            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
        pvas.add_ex(sa, &PRV_LEVEL_FLDS);
        pvas.pvid = pvid.clone();
        let mut mi = si.clone();
        pvi.append(&mut mi);
        //let brns = pv_brns.entry(pvid.clone()).or_default();
        //brns.push(sa.aojcd.clone());
        //if let Some(mbs) = brn_mbs.get(&ii) {
            //for mi in mbs.iter() {
                //brns.push(branch_ass[*mi].0.aojcd.clone());
            //}
        //}
    }
    let mut prv_brn_ass = prv3_hm.into_values().collect();

    prv_sum_1(prv_sub_ass, &mut prv_brn_ass, ass)?;

    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    Ok(prv_brn_ass)
}

pub fn sum_stage_prv_brn2(
    //coreno: usize,
    ass: &Assumption,
    prv_sub_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
    branch_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
    //brn_mbs: &HashMap<usize,Vec<usize>>,
) -> Result<(Vec<(PeaAssVar,Vec<usize>)>,HashMap<String,Vec<String>>), Box<dyn Error>> {

    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    let mut pv_brns = HashMap::<String,Vec<String>>::new();
    let mut prv3_hm = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
    for (_ii,(sa,si)) in branch_ass.iter().enumerate() {
        let pvid = sa.pvid.to_string();
        let (pvas, pvi) = prv3_hm
            .entry(pvid.clone())
            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
        pvas.add_ex(sa, &PRV_LEVEL_FLDS);
        pvas.pvid = pvid.clone();
        let mut mi = si.clone();
        pvi.append(&mut mi);
        let brns = pv_brns.entry(pvid.clone()).or_default();
        brns.push(sa.aojcd.clone());
        //if let Some(mbs) = brn_mbs.get(&ii) {
            //for mi in mbs.iter() {
                //brns.push(branch_ass[*mi].0.aojcd.clone());
            //}
        //}
    }
    let mut prv_brn_ass = prv3_hm.into_values().collect();

    prv_sum_1(prv_sub_ass, &mut prv_brn_ass, ass)?;

    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    Ok((prv_brn_ass, pv_brns))
}

pub fn sum_stage_prv_brn(
    //coreno: usize,
    ass: &Assumption,
    prv_sub_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
    branch_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
) -> Result<(Vec<(PeaAssVar,Vec<usize>)>, HashMap<String,Vec<String>>), Box<dyn Error>> {

    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    let mut pv_brns = HashMap::<String,Vec<String>>::new();
    let mut prv3_hm = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
    for (sa,si) in branch_ass.iter() {
        let pvid = sa.pvid.to_string();
        let (pvas, pvi) = prv3_hm
            .entry(pvid.clone())
            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
        pvas.add_ex(sa, &PRV_LEVEL_FLDS);
        pvas.pvid = pvid.clone();
        let mut mi = si.clone();
        pvi.append(&mut mi);
        let brns = pv_brns.entry(pvid.clone()).or_default();
        brns.push(sa.aojcd.clone());
    }
    let tik = std::time::SystemTime::now();

    /*
    let a_prv2_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar,Vec<usize>)>::new()));
    {
        std::thread::scope(|s| {
            let psz = branch_ass.len().div_ceil(coreno);
            for vsa in branch_ass.chunks_mut(psz) {
                let c_prv2_hm = a_prv2_hm.clone();
                s.spawn(move || {
                    let mut prv2_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                    for (sa, si) in vsa {
                        let pvid = sa.pvid.to_string();
                        let (pvas, pvi) = prv2_m
                            .entry(pvid.clone())
                            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                        pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        pvas.pvid = pvid.clone();
                        let mut mi = si.clone();
                        pvi.append(&mut mi);
                    }
                    if let Ok(mut prv_hm) = c_prv2_hm.lock() {
                        for (_k,(prv, pvi)) in prv2_m.iter_mut() {
                            let pvid = prv.pvid.to_string();
                            let (pvas, pvis) = prv_hm
                                .entry(pvid.clone())
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            pvas.pvid = prv.pvid.clone();
                            let mut mi = pvis.clone();
                            pvi.append(&mut mi);
                        }
                    }
                });
            }
        });
    }
    let prv2_hm = a_prv2_hm.lock().unwrap().clone();
    drop(a_prv2_hm);
    //let mut prv2ass: Vec<_> = prv2_hm.into_values().collect();

    //prv_sub_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,

    let mut prv_brn_ass: Vec<(PeaAssVar,Vec<usize>)>;
    prv_brn_ass = prv2_hm.into_values().collect();
    */

    let mut prv_brn_ass = prv3_hm.into_values().collect();

    prv_sum_1(prv_sub_ass, &mut prv_brn_ass, ass)?;

    let se = tik.elapsed().unwrap().as_secs();
    println!("PRV:{} - {se}sec", prv_brn_ass.len());
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    Ok((prv_brn_ass, pv_brns))
}

pub fn prv_sum_1(
    prv_sub_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
    prv_brn_ass: &mut Vec<(PeaAssVar,Vec<usize>)>,
    ass: &Assumption,
) -> Result<(), Box<dyn Error>> {
//) -> Result<Vec<(PeaAssVar,Vec<usize>)>, Box<dyn Error>> {

    let mut prv_sub_i = HashMap::<String,usize>::new();
    for (i,(ass,_)) in prv_sub_ass.iter().enumerate() {
        if ass.pvid.is_empty() { continue; }
        prv_sub_i.insert(ass.pvid.to_string(), i);
    }
    let mut prv_brn_i = HashMap::<String,usize>::new();
    for (i,(ass,_)) in prv_brn_ass.iter().enumerate() {
        if ass.pvid.is_empty() { continue; }
        prv_brn_i.insert(ass.pvid.to_string(), i);
    }

    //prv_sub_ass.sort_by(|b,a| a.0.pvid.cmp(&b.0.pvid));
    //prv_brn_ass.sort_by(|b,a| a.0.pvid.cmp(&b.0.pvid));
    //prv_sub_ass.sort_by(|a,b| a.pvid.cmp(&b.pvid));
    //prv2ass.sort_by(|a,b| a.pvid.cmp(&b.pvid));
    println!("============  PROVINCE 1 VS 2 = {} - {}  ===========", prv_sub_ass.len(), prv_brn_ass.len());

    for (pv1,iu1) in prv_brn_i.iter() {
        if let Some(iu0) = prv_sub_i.get(pv1) {
            let vas1 = &mut prv_brn_ass[*iu1].0;
            let vas0 = &mut prv_sub_ass[*iu0].0;
            if vas1.pvid!=vas0.pvid { println!(" ERROR prv_sum_1 #01 ======  {} {}", vas1.pvid, vas0.pvid); };
            vas1.v[VarType::BessMWh as usize].v = vas0.v[VarType::BessMWh as usize].v;
            //println!("!!! BESS {} = {}", vas1.pvid, vas0.v[VarType::BessMWh as usize].v);
            vas1.v[VarType::ChgStnCap as usize].v = vas1.v[VarType::ChgStnCapTr as usize].v;
            vas1.v[VarType::ChgStnSell as usize].v = vas1.v[VarType::ChgStnSellTr as usize].v;
            vas1.v[VarType::MvPowSatTr as usize].v = vas1.v[VarType::MaxPosPowSub as usize].v / z2o(vas1.v[VarType::SubPowCap as usize].v);
            vas1.v[VarType::MvVspp as usize].v = vas1.v[VarType::VsppMv as usize].v;
            vas1.v[VarType::HvSpp as usize].v = vas1.v[VarType::SppHv as usize].v;
            vas1.v[VarType::SmallSell as usize].v = vas1.v[VarType::SmallSellTr as usize].v;
            vas1.v[VarType::LargeSell as usize].v = vas1.v[VarType::LargeSellTr as usize].v;
            vas1.v[VarType::UnbalPow as usize].v = vas1.v[VarType::UnbalPowTr as usize].v;
            vas1.v[VarType::TpaZone.tousz()].v /= z2o(vas1.v[VarType::NoTr.tousz()].v);
            let _ = ass_calc(vas1, ass);
        }
    }
    let mut pv2as_mx = PeaAssVar::from(0u64);
    for (pvas,_) in prv_brn_ass.iter() {
        pv2as_mx.max(pvas);
    }
    let mut pv2as_no = prv_brn_ass.clone();
    for (pvas,_) in pv2as_no.iter_mut() {
        pvas.nor(&pv2as_mx);
    }
    for ((pvas,_), (pvno,_)) in prv_brn_ass.iter_mut().zip(pv2as_no.iter()) {
        pvas.v[VarType::TpaZone.tousz()].v = pvno.v[VarType::TpaZone.tousz()].v;
        pvas.v[VarType::MaxPosPowSub.tousz()].v = pvno.v[VarType::MaxPosPowSub.tousz()].v;
    }

    let tpa_day_hours = ass.v(TPA_DAY_HOURS);
    let tpa_price_thb = ass.v(TPA_PRICE_THB);
    let tpa_ben_claim = ass.v(TPA_BEN_CLAIM);
    let tpa_year_days = ass.v(TPA_YEAR_DAYS);
    let tpafcs = ass.ve(TPA_FORECAST)?;
    let NumValEnum::Json(tpa_json) = tpafcs else {
        return Err("No TPA forecast data".into());
    };
    let Value::Array(tpa_ary) = tpa_json else {
        return Err("TPA value is not array".into());
    };
    let mut tpa_fa = vec![];
    for va in tpa_ary {
        let Value::Number(n) = va else {
            continue;
        };
        let Some(f) = n.as_f64() else {
            continue;
        };
        tpa_fa.push(f as f32);
    }

    let mut we_tpa = PeaAssVar::from(0u64);
    for (vt, vv) in WE_TPA {
        we_tpa.v[vt.tousz()].v = vv;
    }
    let mut tpa_ad = PeaAssVar::from(0u64);
    let mut pvtpa = prv_sub_ass.clone();
    let _flds = [VarType::TpaFcst.tousz()];
    for ((tpa,_), (prv,_)) in pvtpa.iter_mut().zip(prv_sub_ass.iter_mut()) {
        tpa.weigh(&we_tpa);
        tpa.sum();
        prv.v[VarType::TpaFcst as usize].v = tpa.res;
        tpa_ad.v[VarType::TpaFcst.tousz()].v += prv.v[VarType::TpaFcst.tousz()].v;
        //tpa_ad.add_ex(prv, &flds);
    }

    let mut pv2as_no = prv_brn_ass.clone();
    for ((no,_), (prv,_)) in pv2as_no.iter_mut().zip(prv_brn_ass.iter_mut()) {
        no.nor(&tpa_ad);
        prv.v[VarType::TpaFcst.tousz()].v = no.v[VarType::TpaFcst.tousz()].v;
    }

    for (pvas,_) in prv_brn_ass.iter_mut() {
        let rat = pvas.v[VarType::TpaFcst.tousz()].v;
        let mut tpav = vec![];
        for y in tpa_fa.iter().take(15) {
            let y = y * tpa_day_hours * tpa_price_thb * tpa_year_days;
            let y = y * rat;
            let y = y * tpa_ben_claim;
            let y = y * 1_000f32;
            tpav.push(y);
        }
        pvas.vy[VarType::FirTpaThb.tousz()].append(&mut tpav);
        pvas.yr_sum(VarType::FirTpaThb, ass);
    }
    Ok(())
}

pub type PeaAssWithMemb = Vec<(PeaAssVar,Vec<usize>)>;

use std::collections::HashSet;

pub fn sum_stage_brn1_b(
    _coreno: usize,
    _assu: &Assumption,
    brn_ass: &mut PeaAssWithMemb,
) -> Result<(PeaAssWithMemb, HashMap<String,Vec<String>>), Box<dyn Error>> {
    //let brnass1 = brn_ass.clone();
    let (brns, _cd_bri) = get_brn_map()?;

    let mut h_brns_sti = HashSet::<usize>::new();
    for (i,br) in brns.iter().enumerate() {
        if let Some(is) = br.i_stock {
            h_brns_sti.insert(is);
        } else if br.is_prv {
            h_brns_sti.insert(i);
        }
    }
    println!("stock branch no: {}", h_brns_sti.len());
    let mut ass_cd_i = HashMap::<String,usize>::new();
    for (i,(ass,_)) in brn_ass.iter().enumerate() {
        ass_cd_i.insert(ass.aojcd.clone(), i);
    }

    //let mut mn_brns = Vec::<(usize,Vec<usize>)>::new();
    let mut mn_brns = HashMap::<String,Vec<String>>::new();
    //let mut cn = 0;
    for (i,br) in brns.iter().enumerate() {
        if h_brns_sti.contains(&i) && let Some(_assi) = ass_cd_i.get(&br.code) {
            //let ass: (PeaAssVar,Vec<usize>) = brn_ass[*assi].clone();
            mn_brns.insert(br.code.clone(), vec![br.code.clone()]);
            //cn += 1;
            //println!("MAIN: {cn}. {} - {} stok:{}", br.code, br.name, br.has_stock);
        }
    }

    println!("######### 1");
    //let mut cn = 0;
    for (i,br) in brns.iter().enumerate() {
        if !h_brns_sti.contains(&i) {
            if let Some(_assi) = ass_cd_i.get(&br.code) {
                if let Some(is) = br.i_stock {
                    let pacd = brns[is].code.clone();
                    if let Some(mbs) = mn_brns.get_mut(&pacd) {
                        mbs.push(br.code.clone());
                    } else {
                        println!("  BRANCH CHECK #1 {pacd}");
                    }
                } else if let Some(is) = br.i_prov {
                    let pacd = brns[is].code.clone();
                    if let Some(mbs) = mn_brns.get_mut(&pacd) {
                        mbs.push(br.code.clone());
                    } else {
                        println!("  BRANCH CHECK #2 {pacd}");
                    }
                } else {
                    println!("  BRANCH CHECK #3");
                }
            } else {
                println!("   BRANCH CHECK #4 {}", br.code);

            }
            //let ass: (PeaAssVar,Vec<usize>) = brn_ass[*assi].clone();
            //mn_brns.push(ass);
            //cn += 1;
            //println!("SUB: {cn}. {} - {}", br.code, br.name);
            /*
            if let Some(is) = br.i_stock && let Some(assv) = mn_brns.get_mut(&br.code) {
                assv.push(br.code.clone());
            } else {
                if let Some(is) = br.i_prov && let Some(assv) = mn_brns.get_mut(&br.code) {
                    assv.push(br.code.clone());
                } else {
                    println!("=========== Sub Error {i}. {} sto:{:?} pv:{:?}", br.name, br.i_stock, br.i_prov);
                }
            }
            */
        }
    }

    println!("######### 2");

    let mut mn_assv = Vec::<(PeaAssVar,Vec<usize>)>::new();
    for (k,cds) in mn_brns.iter() {
        let mut assa = (PeaAssVar::from(0u64),vec![]);
        let Some(assi) = ass_cd_i.get(k) else {
            println!("  BRANCH ERROR #5 {}", k);
            continue;
        };
        assa.0.aojcd = brn_ass[*assi].0.aojcd.clone();
        assa.0.pvid = brn_ass[*assi].0.pvid.clone();
        for cd in cds.iter() {
            let Some(assi) = ass_cd_i.get(cd) else {
                continue;
            };
            let assb = brn_ass[*assi].clone();
            assa.0.add_ex(&assb.0, &SUB_LEVEL_FLDS);
        }
        mn_assv.push(assa);
    }
    /*
    for ass in brn_ass.iter() {}
    let mut brnis = mn_brns.keys().cloned().collect::<Vec<usize>>();
    brnis.sort();
    let mut mn_assv = Vec::<(PeaAssVar,Vec<usize>)>::new();
    //let mut cn = 0;
    for ki in brnis.iter() {
        if let Some(assis) = mn_brns.get(ki) && !assis.is_empty() {
            //cn += 1;
            //println!("{cn}. {ki} '{}' l:{}", brns[*ki].name, assis.len());
            let mut assis = assis.clone();
            let ass0 = assis.pop().unwrap();
            let mut assa = brn_ass[ass0].clone();
            assa.0.aojcd = brns[*ki].code.clone();
            while let Some(ass) = assis.pop() {
                let assb = brn_ass[ass].clone();
                assa.0.add_ex(&assb.0, &SUB_LEVEL_FLDS);
            }
            mn_assv.push(assa);
        } else {
            println!("ERROR ================= brn :{}", ki);
        }
    }
    */
    println!("======= BRANCH MAIN {}", mn_assv.len());
    Ok((mn_assv, mn_brns))
}

/*
pub fn sum_stage_brn1(
    _coreno: usize,
    _assu: &Assumption,
    brn_ass: &mut PeaAssWithMemb,
) -> Result<(PeaAssWithMemb, HashMap<usize,Vec<usize>>), Box<dyn Error>> {
    //let brnass1 = brn_ass.clone();
    let (brns, _cd_bri) = get_brn_map()?;

    let mut h_brns_sti = HashSet::<usize>::new();
    for (i,br) in brns.iter().enumerate() {
        if let Some(is) = br.i_stock {
            h_brns_sti.insert(is);
        } else if br.is_prv {
            h_brns_sti.insert(i);
        }
    }
    println!("stock branch no: {}", h_brns_sti.len());
    let mut ass_cd_i = HashMap::<String,usize>::new();
    for (i,(ass,_)) in brn_ass.iter().enumerate() {
        ass_cd_i.insert(ass.aojcd.clone(), i);
    }

    //let mut mn_brns = Vec::<(usize,Vec<usize>)>::new();
    let mut mn_brns = HashMap::<usize,Vec<usize>>::new();
    //let mut cn = 0;
    for (i,br) in brns.iter().enumerate() {
        if h_brns_sti.contains(&i) && let Some(assi) = ass_cd_i.get(&br.code) {
            //let ass: (PeaAssVar,Vec<usize>) = brn_ass[*assi].clone();
            mn_brns.insert(i, vec![*assi]);
            //cn += 1;
            //println!("MAIN: {cn}. {} - {} stok:{}", br.code, br.name, br.has_stock);
        }
    }

    //let mut cn = 0;
    for (i,br) in brns.iter().enumerate() {
        if !h_brns_sti.contains(&i) && let Some(assi) = ass_cd_i.get(&br.code) {
            //let ass: (PeaAssVar,Vec<usize>) = brn_ass[*assi].clone();
            //mn_brns.push(ass);
            //cn += 1;
            //println!("SUB: {cn}. {} - {}", br.code, br.name);
            if let Some(is) = br.i_stock && let Some(assv) = mn_brns.get_mut(&is) {
                assv.push(*assi);
            } else {
                if let Some(is) = br.i_prov && let Some(assv) = mn_brns.get_mut(&is) {
                    assv.push(*assi);
                } else {
                    println!("=========== Sub Error {i}. {} sto:{:?} pv:{:?}", br.name, br.i_stock, br.i_prov);
                }
            }
        }
    }
    let mut brnis = mn_brns.keys().cloned().collect::<Vec<usize>>();
    brnis.sort();
    let mut mn_assv = Vec::<(PeaAssVar,Vec<usize>)>::new();
    //let mut cn = 0;
    for ki in brnis.iter() {
        if let Some(assis) = mn_brns.get(ki) && !assis.is_empty() {
            //cn += 1;
            //println!("{cn}. {ki} '{}' l:{}", brns[*ki].name, assis.len());
            let mut assis = assis.clone();
            let ass0 = assis.pop().unwrap();
            let mut assa = brn_ass[ass0].clone();
            assa.0.aojcd = brns[*ki].code.clone();
            while let Some(ass) = assis.pop() {
                let assb = brn_ass[ass].clone();
                assa.0.add_ex(&assb.0, &SUB_LEVEL_FLDS);
            }
            mn_assv.push(assa);
        } else {
            println!("ERROR ================= brn :{}", ki);
        }
    }
    println!("======= BRANCH MAIN {}", mn_assv.len());
    Ok((mn_assv,mn_brns))
}
*/


