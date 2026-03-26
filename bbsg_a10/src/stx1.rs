use crate::dcl::AojObj;
use crate::dcl::ProcEngine;
use crate::dcl::*;
use crate::utl::get_tr_volta;
use crate::utl::trf_kva_2_kw;
use regex::Regex;
use sglib04::geo4::GPPS;
use std::collections::HashMap;
use std::error::Error;

///
/// g0 = ProcEngine::prep1();
///
/// evpv: p13_ev_distr(&EV_PRV_ADJ_1)
///
/// sbif: sub_inf()
///
/// lp23: p03_load_lp("2023")
///
/// lp24: p03_load_lp("2024")
///
/// for (sb, sf) in &g0.sbif {
///
///   for id in &aids {
///
/// dnm = "/mnt/e/CHMBACK/pea-data/c01_pea";
///
/// bin: `Vec<u8>` = bincode::encode_to_vec(&pea, bincode::config::standard())
///
/// write(format!("{dnm}/000_pea.bin"), bin)
///
/// bin: `Vec<u8>` = bincode::encode_to_vec(&sb, bincode::config::standard())
///
/// std::fs::write(format!("{dnm}/{}.bin", sb.sbid), bin)
///
pub fn stage_01() -> Result<(), Box<dyn Error>> {
    //let dnm = "/mnt/e/CHMBACK/pea-data/c01_pea";
    println!("===== STAGE 1 =====");
    let g0 = ProcEngine::prep5();
    let pea = c01_chk_01_01(DNM, &g0)?;
    c01_chk_01_02(&pea, DNM, &g0)?;

    let bin: Vec<u8> = bincode::encode_to_vec(&pea, bincode::config::standard()).unwrap();
    std::fs::write(format!("{DNM}/000_pea.bin"), bin).unwrap();
    println!("write2 to 000_pea.bin");

    Ok(())
}

pub fn c01_chk_01_01(dnm: &str, g0: &ProcEngine) -> Result<Pea, Box<dyn Error>> {
    std::fs::create_dir_all(dnm)?;
    let mut pea = Pea::default();
    for (sb, sf) in &g0.sbif {
        let ar = sf.arid.to_string();
        let ar_e = pea.aream.entry(ar).or_insert_with(|| PeaArea {
            arid: sf.arid.to_string(),
            ..Default::default()
        });
        let pv_e = ar_e
            .provm
            .entry(sf.prov.to_string())
            .or_insert_with(|| PeaProv {
                pvnm: sf.prov.to_string(),
                ..Default::default()
            });

        let Some(ev) = g0.evpv.get(&sf.prov) else {
            continue;
        };
        let Some(gpp) = GPPS.get(&sf.prov) else {
            continue;
        };
        pv_e.evpc = ev.ev_pc;
        pv_e.gppv = *gpp as f32;

        let _sb_e = pv_e.subm.entry(sb.to_string()).or_insert_with(|| PeaSub {
            sbid: sb.to_string(),
            name: sf.name.clone(),
            enam: sf.enam.clone(),
            area: sf.area.clone(),
            arid: sf.arid.clone(),
            volt: sf.volt.clone(),
            cate: sf.cate.clone(),
            egat: sf.egat.clone(),
            state: sf.state.clone(),
            conf: sf.conf.clone(),
            trax: sf.trax.clone(),
            mvax: sf.mvax.clone(),
            feed: sf.feed.clone(),
            feno: sf.feno,
            feeders: sf.feeders.clone(),
            trxn: sf.trxn,
            mvxn: sf.mvxn,
            prov: sf.prov.clone(),
            ..Default::default()
        });
    }

    let bin: Vec<u8> = bincode::encode_to_vec(&pea, bincode::config::standard()).unwrap();
    std::fs::write(format!("{dnm}/000_pea.bin"), bin).unwrap();
    println!("write to 000_pea.bin");

    Ok(pea)
}

use std::sync::Arc;
use std::sync::Mutex;

pub fn c01_chk_01_02(pea: &Pea, dnm: &str, g00: &ProcEngine) -> Result<(), Box<dyn Error>> {
    let smrt = Regex::new(r"[12].*").unwrap();
    //let aream = pea.aream.clone();
    //let mut aids: Vec<_> = pea.aream.keys().collect();
    //aids.sort();

    let a_g0 = Arc::new(Mutex::new(g00.clone()));
    let a_aoj_sbv = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));

    let a_sbs = Arc::new(Mutex::new(HashMap::<String, PeaSub>::new()));

    // ======== first round for adding sub
    std::thread::scope(|s| {
        for (aid, ar) in &pea.aream {
            let c_g0 = a_g0.clone();
            let c_aoj_sbv = a_aoj_sbv.clone();
            let provm = ar.provm.clone();
            let smrt = smrt.clone();
            let c_sbs = a_sbs.clone();
            let subs0 = ProcEngine::subs0(&aid);
            s.spawn(move || {
                let mut pids: Vec<_> = provm.keys().collect();
                pids.sort();
                for id in &pids {
                    let pid = id.to_string();
                    let Some(pr) = provm.get(&pid) else {
                        continue;
                    };
                    // province
                    // Car registration
                    let ev = if let Ok(g0) = c_g0.lock() {
                        let Some(ev) = g0.evpv.get(&pid) else {
                            continue;
                        };
                        ev.clone()
                    } else {
                        continue;
                    };
                    let Some(gpp) = GPPS.get(&pid) else {
                        continue;
                    };
                    println!("  p:{id}");
                    println!("    ev rt: {}", ev.ev_pc);
                    println!("    gpp : {}", gpp);
                    //let mut subm = pr.subm.clone();
                    let mut sids: Vec<_> = pr.subm.keys().collect();
                    sids.sort();
                    for id in &sids {
                        let sbid = id.to_string();
                        let sid = id.to_string();
                        let Some(sb) = pr.subm.get(&sid) else {
                            println!("          ====================== NO1 {sid}");
                            continue;
                        };
                        let is = subs0
                            .iter()
                            .enumerate()
                            .filter(|(_, r)| r.sbid == sbid)
                            .map(|(i, _)| i)
                            .collect::<Vec<_>>();
                        //println!("    s:{id} is:{is:?}");
                        if is.is_empty() {
                            println!("          ====================== NO2 {sid}");
                            continue;
                        }
                        let si = is[0];
                        let sub = &subs0[si];
                        //////////////////////////////////////////////
                        //  Substation Info
                        let mut sb = sb.clone();
                        sb.sbtp = sub.conf.to_string();
                        sb.n1d_s = sub.n1d_s;
                        sb.n1d_f = sub.n1d_f;
                        // substation info
                        if let Ok(g0) = c_g0.lock() {
                            if let Some(slp) = g0.lp23.get(&sbid) {
                                sb.lp_rep_23 = slp.clone();
                            }
                            if let Some(slp) = g0.lp24.get(&sbid) {
                                sb.lp_rep_24 = slp.clone();
                            }
                        }
                        println!("       ===== write to {}.bin", sb.sbid);
                        if let Ok(mut sbs) = c_sbs.lock() {
                            sbs.insert(sbid, sb);
                        }
                        ////////////////////////////////////////////
                        ////////////////////////////////////////////
                    }
                }
            });
        }
    });
    let aoj_sbv = a_aoj_sbv.lock().unwrap();
    let aoj_sbv = aoj_sbv.clone();
    let bin: Vec<u8> = bincode::encode_to_vec(&aoj_sbv, bincode::config::standard()).unwrap();
    std::fs::write(format!("{dnm}/aoj_sbv.bin"), bin).unwrap();
    println!("       ===== write to aoj_sbv.bin");

    Ok(())
}

pub fn stage_x1() -> Result<(), Box<dyn Error>> {
    //let dnm = "/mnt/e/CHMBACK/pea-data/c01_pea";
    println!("===== STAGE 1 =====");
    let g0 = ProcEngine::prep5();

    std::fs::create_dir_all(DNM)?;
    let mut pea = Pea::default();
    for (sb, sf) in &g0.sbif {
        let ar = sf.arid.to_string();
        let ar_e = pea.aream.entry(ar).or_insert_with(|| PeaArea {
            arid: sf.arid.to_string(),
            ..Default::default()
        });
        let pv_e = ar_e
            .provm
            .entry(sf.prov.to_string())
            .or_insert_with(|| PeaProv {
                pvnm: sf.prov.to_string(),
                ..Default::default()
            });

        let Some(ev) = g0.evpv.get(&sf.prov) else {
            continue;
        };
        let Some(gpp) = GPPS.get(&sf.prov) else {
            continue;
        };
        pv_e.evpc = ev.ev_pc;
        pv_e.gppv = *gpp as f32;

        let _sb_e = pv_e.subm.entry(sb.to_string()).or_insert_with(|| PeaSub {
            sbid: sb.to_string(),
            name: sf.name.clone(),
            enam: sf.enam.clone(),
            area: sf.area.clone(),
            arid: sf.arid.clone(),
            volt: sf.volt.clone(),
            cate: sf.cate.clone(),
            egat: sf.egat.clone(),
            state: sf.state.clone(),
            conf: sf.conf.clone(),
            trax: sf.trax.clone(),
            mvax: sf.mvax.clone(),
            feed: sf.feed.clone(),
            feno: sf.feno,
            feeders: sf.feeders.clone(),
            trxn: sf.trxn,
            mvxn: sf.mvxn,
            prov: sf.prov.clone(),
            ..Default::default()
        });
    }

    let bin: Vec<u8> = bincode::encode_to_vec(&pea, bincode::config::standard()).unwrap();
    let fnm = format!("{DNM}/000_pea.bin");
    println!("write to {fnm}");
    std::fs::write(fnm, bin).unwrap();

    let smrt = Regex::new(r"[12].*").unwrap();
    //let aream = pea.aream.clone();
    let mut aids: Vec<_> = pea.aream.keys().collect();
    aids.sort();

    //let a_sbs = Arc::new(Mutex::new(vec![]));
    //let mut aoj_sbv = HashMap::<String, Vec<String>>::new();
    //let eg = ProcEngine::prep2(&id);
    //let a_eg = Arc::new(Mutex::new(eg));
    //let a_eg = Arc::new(Mutex::new(ProcEngine::prep2(&id)));
    //let provm = ar.provm.clone();

    // =========================================================== BEGIN
    // =========================================================== BEGIN
    // =========================================================== BEGIN
    //let c_eg = a_eg.clone();
    //let c_aoj_sbv = a_aoj_sbv.clone();
    //let aream = pea.aream.clone();
    //let smrt = smrt.clone();
    //let c_sbs = a_sbs.clone();
    //let mut pids: Vec<_> = provm.keys().collect();
    //pids.sort();
    //for id in &pids {
    //let Some(pr) = provm.get(&pid) else {
    //continue;
    //};
    //let mut subm = pr.subm.clone();
    //let mut sids: Vec<_> = pr.subm.keys().collect();
    //sids.sort();
    //for id in &sids {
    //let sid = id.to_string();
    //let Some(sb) = pr.subm.get(&sid) else {
    //println!("          ====================== NO1 {sid}");
    //continue;
    //};

    let a_g0 = Arc::new(Mutex::new(g0));
    //let a_g0 = Arc::new(Mutex::new(g0.clone()));
    let a_aoj_sbv = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));
    //let mut handles = vec![];
    let ar_sbs = HashMap::<String, Vec<PeaSub>>::new();
    let a_sbs = Arc::new(Mutex::new(ar_sbs));

    {
        std::thread::scope(|s| {
            for (aid, ar) in &pea.aream {
                println!("ar:{aid}");
                let aid = aid.to_string();
                let c_g0 = a_g0.clone();
                let provm = ar.provm.clone();
                let c_sbs = a_sbs.clone();

                s.spawn(move || {
                    let subs0 = ProcEngine::subs0(&aid);
                    for (pid, pr) in &provm {
                        let pid = pid.to_string();
                        // province
                        // Car registration
                        let ev = if let Ok(g0) = c_g0.lock() {
                            let Some(ev) = g0.evpv.get(&pid) else {
                                continue;
                            };
                            ev.clone()
                        } else {
                            continue;
                        };
                        let Some(gpp) = GPPS.get(&pid) else {
                            continue;
                        };
                        println!("  p:{pid}");
                        println!("    ev rt: {}", ev.ev_pc);
                        println!("    gpp : {}", gpp);
                        for (sid, sb) in &pr.subm {
                            let sbid = sid.to_string();
                            let is = subs0
                                .iter()
                                .enumerate()
                                .filter(|(_, r)| r.sbid == sbid)
                                .map(|(i, _)| i)
                                .collect::<Vec<_>>();
                            //println!("    s:{id} is:{is:?}");
                            if is.is_empty() {
                                println!("          ====================== NO2 {sbid}");
                                continue;
                            }
                            let si = is[0];
                            let sub = &subs0[si];
                            //////////////////////////////////////////////
                            //  Substation Info
                            let mut sb = sb.clone();
                            sb.sbtp = sub.conf.to_string();
                            sb.n1d_s = sub.n1d_s;
                            sb.n1d_f = sub.n1d_f;
                            // substation info
                            if let Ok(g0) = c_g0.lock() {
                                if let Some(slp) = g0.lp23.get(&sbid) {
                                    sb.lp_rep_23 = slp.clone();
                                }
                                if let Some(slp) = g0.lp24.get(&sbid) {
                                    sb.lp_rep_24 = slp.clone();
                                }
                            }
                            println!("       ===== ADD to SUB {}.bin", sb.sbid);
                            if let Ok(mut sbs) = c_sbs.lock() {
                                let aid = aid.to_string();
                                let sba = sbs.entry(aid).or_insert(Vec::<PeaSub>::new());
                                sba.push(sb);
                            }
                            //////////////////////////////////////////////////
                            //////////////////////////////////////////////////
                        }
                    }
                });
            } // end area loop
        });
    }
    let Ok(mut ar_sbs) = a_sbs.lock() else {
        println!("======================== ERROR 1 ======================== ");
        return Err("no".into());
    };
    println!("AREA CNT: {}", ar_sbs.len());

    std::thread::scope(|s| {
        for (aid, sbs) in ar_sbs.iter_mut() {
            //println!("{aid} - {}", sbs.len());
            s.spawn(move || {
                let subs0 = ProcEngine::subs0(aid);
                let vssb0 = ProcEngine::vssb0(aid);
                let spsb0 = ProcEngine::spsb0(aid);
                let resb0 = ProcEngine::resb0(aid);
                for sb in sbs.iter_mut() {
                    let sbid = sb.sbid.to_string();
                    let is = subs0
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.sbid == sbid)
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();
                    //println!("    s:{id} is:{is:?}");
                    if is.is_empty() {
                        println!("          ====================== NO2 {sbid}");
                        continue;
                    }
                    let si = is[0];
                    let mut vspps = vec![];
                    let vsp = &vssb0[si];
                    if !vsp.is_empty() {
                        for pi in vsp {
                            vspps.push(*pi);
                        }
                    }
                    sb.vspps = vspps;

                    let mut spps = vec![];
                    let spp = &spsb0[si];
                    if !spp.is_empty() {
                        for pi in spp {
                            spps.push(*pi);
                        }
                    }
                    sb.spps = spps;

                    let mut repls = vec![];
                    let repl = &resb0[si];
                    if !repl.is_empty() {
                        for pi in repl {
                            repls.push(*pi);
                        }
                    }
                    sb.repls = repls;
                }
                println!("SPAWN 1 : {aid} - {}", sbs.len());
            });
        }
    });

    std::thread::scope(|s| {
        for (aid, sbs) in ar_sbs.iter_mut() {
            //println!("{aid} - {}", sbs.len());
            s.spawn(move || {
                println!("SPAWN 2 : {aid} - {} BEGIN", sbs.len());
                let subs0 = ProcEngine::subs0(aid);
                let aotr0 = ProcEngine::aotr0(aid);
                let amtr0 = ProcEngine::amtr0(aid);
                let mutr0 = ProcEngine::mutr0(aid);
                let zntr0 = ProcEngine::zntr0(aid);
                let sotr0 = ProcEngine::sotr0(aid);
                let ctrs0 = ProcEngine::ctrs0(aid);
                let votr0 = ProcEngine::votr0(aid);
                let vols0 = ProcEngine::vols0(aid);
                let cmts0 = ProcEngine::cmts0(aid);
                for sb in sbs.iter_mut() {
                    let sbid = sb.sbid.to_string();
                    let is = subs0
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.sbid == sbid)
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();
                    if is.is_empty() {
                        println!("          ====================== NO2 {sbid}");
                        continue;
                    }
                    let si = is[0];
                    let sub = &subs0[si];

                    let mut fds = sub.feed.keys().collect::<Vec<_>>();
                    fds.sort();
                    let mut aoj_tr = HashMap::<usize, usize>::new();
                    for fid in fds {
                        let fid = fid.to_string();
                        let Some(tis) = sub.feed.get(&fid) else {
                            continue;
                        };
                        let fd = sb.feedm.entry(fid.to_string()).or_insert_with(|| PeaFeed {
                            fdid: fid.to_string(),
                            ..Default::default()
                        });
                        for ti in tis {
                            let tr = &ctrs0[*ti];
                            let t1d = tr.n1d;
                            let trs = fd.tranm.entry(t1d).or_insert_with(|| PeaTrans {
                                trid: tr.trid.to_string(),
                                pea: tr.pea.to_string(),
                                n1d: tr.n1d,
                                n1d_f: tr.n1d_f,
                                ix: tr.ix,
                                lix: tr.lix,
                                mts: tr.mts.clone(),
                                ..Default::default()
                            });
                            ////////////// AojObj
                            trs.aojs = aotr0[*ti].clone();
                            for ai in &trs.aojs {
                                let ai = *ai;
                                if let Some(cn) = aoj_tr.get_mut(&ai) {
                                    *cn += 1;
                                } else {
                                    aoj_tr.insert(ai, 1);
                                }
                            }
                            trs.amps = amtr0[*ti].clone();
                            trs.muns = mutr0[*ti].clone();
                            trs.zons = zntr0[*ti].clone();
                            trs.sols = if sotr0.len() > *ti {
                                sotr0[*ti].clone()
                            } else {
                                Vec::<_>::new()
                            };
                            /*
                            let tcm = &cmts0[tr.ix];
                            let Some(ow) = &tcm.tr_own else {
                                continue;
                            };
                            trs.own = ow.clone();
                            trs.vols = votr0[*ti].clone();
                            //votr[*ti]
                            let vos = &votr0[*ti];
                            let (vopw, vose) = if let Some(vi) = vos.iter().next() {
                                let vo = &vols0[*vi];
                                let mut pow = 0.0;
                                for (pw, no) in &vo.chgr {
                                    pow += (pw * no) as f32;
                                }
                                let mut sel = 0.0;
                                //println!("VOL: {:?}", vo.stno);
                                for (_ym, am) in &vo.sell {
                                    sel += am;
                                    //println!("  {ym} {am}");
                                }
                                (pow, sel)
                            } else {
                                (0.0, 0.0)
                            };
                            trs.vopw = vopw;
                            trs.vose = vose;
                            trs.from_cmt(tcm);
                            //println!("        trs: {}", trs.n1d);

                            let Some(kv) = &tcm.tr_kva else {
                                continue;
                            };
                            if *kv == 0.0 {
                                continue;
                            }
                            trs.kw = trf_kva_2_kw(*kv);
                            */
                        }
                    }
                }
                println!("SPAWN 2 : {aid} - {} END", sbs.len());
            });
        }
    });

    std::thread::scope(|s| {
        for (aid, sbs) in ar_sbs.iter_mut() {
            //println!("{aid} - {}", sbs.len());
            let smrt = smrt.clone();
            let c_aoj_sbv = a_aoj_sbv.clone();
            s.spawn(move || {
                println!("SPAWN 3 : {aid} - {} BEGIN", sbs.len());
                let subs0 = ProcEngine::subs0(aid);
                let ctrs0 = ProcEngine::ctrs0(aid);
                let cmts0 = ProcEngine::cmts0(aid);
                let m2bs0 = ProcEngine::m2bs0(aid);
                let bils0 = ProcEngine::bils0(aid);
                let aojs0 = ProcEngine::aojs0(aid);
                for sb in sbs.iter_mut() {
                    let sbid = sb.sbid.to_string();
                    let is = subs0
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.sbid == sbid)
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();
                    if is.is_empty() {
                        println!("          ====================== NO2 {sbid}");
                        continue;
                    }
                    let si = is[0];
                    let sub = &subs0[si];

                    let mut fds = sub.feed.keys().collect::<Vec<_>>();
                    fds.sort();
                    let mut aoj_tr = HashMap::<usize, usize>::new();
                    for fid in fds {
                        let fid = fid.to_string();
                        let Some(tis) = sub.feed.get(&fid) else {
                            continue;
                        };
                        let fd = sb.feedm.entry(fid.to_string()).or_insert_with(|| PeaFeed {
                            fdid: fid.to_string(),
                            ..Default::default()
                        });
                        for ti in tis {
                            let tr = &ctrs0[*ti];
                            let t1d = tr.n1d;
                            let trs = fd.tranm.entry(t1d).or_insert_with(|| PeaTrans {
                                trid: tr.trid.to_string(),
                                pea: tr.pea.to_string(),
                                n1d: tr.n1d,
                                n1d_f: tr.n1d_f,
                                ix: tr.ix,
                                lix: tr.lix,
                                mts: tr.mts.clone(),
                                ..Default::default()
                            });
                            ////////////// AojObj
                            for mi in &tr.mts {
                                let mt = &cmts0[*mi];
                                let mb = &m2bs0[*mi];
                                if mb.is_empty() {
                                    continue;
                                }
                                let bl = &bils0[mb[0]];
                                let tp = if smrt.captures(bl.rate.as_str()).is_some() {
                                    MeterAccType::Small
                                } else {
                                    MeterAccType::Large
                                };
                                let mut met = PeaMeter::default();
                                met.from_cmt(mt);
                                met.from_bil(bl);
                                met.met_type = tp;
                                trs.mets.push(met);
                            } // end mi
                        }
                    }
                    let mut aojs: Vec<(usize, usize)> =
                        aoj_tr.into_iter().map(|(k, v)| (v, k)).collect();
                    aojs.sort_by(|a, b| b.0.cmp(&a.0));
                    let mut aojv = Vec::<AojObj>::new();
                    for (v, ai) in aojs {
                        let ao = &aojs0[ai];
                        let code = ao.code.clone().unwrap_or("".to_string());
                        let sht_name = ao.sht_name.clone().unwrap_or("".to_string());
                        let office = ao.office.clone().unwrap_or("".to_string());
                        let pea = ao.pea.clone().unwrap_or("".to_string());
                        let aoj_sz = ao.aoj_sz.clone().unwrap_or("".to_string());
                        let reg = ao.reg.clone().unwrap_or("".to_string());
                        let name = ao.name.clone().unwrap_or("".to_string());
                        let level = ao.level.unwrap_or(0f32);
                        let trcn = v;
                        //aojv.push((eg.aojs[ai].clone(), v));
                        let aoj = AojObj {
                            code: code.to_string(),
                            sht_name: sht_name.to_string(),
                            office: office.to_string(),
                            pea: pea.to_string(),
                            aoj_sz: aoj_sz.to_string(),
                            reg: reg.to_string(),
                            name: name.to_string(),
                            level,
                            trcn,
                        };
                        aojv.push(aoj);
                        {
                            let mut aoj_sbv = c_aoj_sbv.lock().unwrap();
                            let aojsb = aoj_sbv.entry(code.to_string()).or_default();
                            aojsb.push(sbid.to_string());
                        }
                    } // end aoj loop
                    sb.aojv = aojv;
                }
                println!("SPAWN 3 : {aid} - {} END", sbs.len());
            });
        }
    });

    let aoj_sbv = a_aoj_sbv.lock().unwrap();
    let aoj_sbv = aoj_sbv.clone();
    let bin: Vec<u8> = bincode::encode_to_vec(&aoj_sbv, bincode::config::standard()).unwrap();
    std::fs::write(format!("{DNM}/aoj_sbv.bin"), bin).unwrap();
    println!("       ===== write to aoj_sbv.bin");

    Ok(())
}
