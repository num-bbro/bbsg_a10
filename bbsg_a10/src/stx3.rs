use crate::asm::ASM::*;
use crate::dcl::Pea;
//use crate::dcl::PeaAssVar;
use crate::dcl::PeaAssVar;
use crate::dcl::PeaSub;
use crate::dcl::ProcEngine;
use crate::dcl::VarType;
use crate::stx2::PRV_LEVEL_FLDS;
use crate::utl4::Archi;
use std::collections::HashMap;
use std::error::Error;

pub fn subass_to_prvass(ac: &Archi) -> Result<(), Box<dyn Error>> {
    let dnm = ac.t(OUTDIR);
    let fnm = format!("{dnm}/000-sbrw.bin");
    println!("fnm:{fnm}");
    let buf = std::fs::read(fnm).unwrap();
    let (subass, _): (Vec<PeaAssVar>, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
    println!("subass: {}", subass.len());

    let mut prv_m = HashMap::<String, PeaAssVar>::new();
    let mut meno = 0.0;
    for sa in subass.iter() {
        let pvid = sa.pvid.to_string();
        if let Some(pvas) = prv_m.get_mut(&pvid) {
            pvas.add_ex(sa, &PRV_LEVEL_FLDS);
        } else {
            prv_m.insert(pvid, sa.clone());
        }
        //let pvas = prv_m.entry(pvid).or_insert_with(|| sa.clone());
        meno += sa.v[VarType::NoMet1PhA.tousz()].v;
        //pvas.add_ex(sa, &PRV_LEVEL_FLDS);
        //pvas.v[VarType::NoMet1PhA.tousz()].v += sa.v[VarType::NoMet1PhA.tousz()].v;
    }
    println!("all me 1: {meno}");
    let mut meno = 0.0;
    for (pid, pvas) in prv_m {
        meno += pvas.v[VarType::NoMet1PhA.tousz()].v;
        println!("{pid} - {}", pvas.v[VarType::NoMet1PhA.tousz()].v);
    }
    println!("all me 2: {meno}");
    Ok(())
}

pub fn proc(ac: &Archi) -> Result<(), Box<dyn Error>> {
    let dnm = ac.t(OUTDIR);
    let fnm = format!("{dnm}/000_pea.bin");
    println!("fnm:{fnm}");
    let buf = std::fs::read(fnm).unwrap();
    let (pea, _): (Pea, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
    let mut pno = 0;
    //let mut pvm = HashMap::<String, String>::new();
    let mut sbm = HashMap::<String, usize>::new();
    let mut tn = 0;
    let mut mn = 0;
    let mut nsn = 0;
    let mut nfn = 0;
    let mut n_ctrs = 0;
    let mut n_cmts = 0;
    let mut n_bils = 0;
    let mut n_m2bs = 0;
    for (aid, ar) in pea.aream.iter() {
        println!("aid:{aid}");
        let eg = ProcEngine::prep7(aid);
        n_ctrs += eg.ctrs.len();
        n_cmts += eg.cmts.len();
        n_bils += eg.bils.len();
        n_m2bs += eg.m2bs.len();
        for (pid, pv) in ar.provm.iter() {
            /*
            if let Some(a0) = pvm.get(pid) {
                println!("  >>>>> pv:{pid} {aid} = {a0}");
            } else {
                pvm.insert(pid.clone(), aid.clone());
                println!("  pv {pid} - {aid}");
            }
            */
            pno += 1;
            let sids: Vec<_> = pv.subm.keys().collect();
            println!("  {pid} - {}", sids.len());
            for sid in sids.iter() {
                //println!("    {sid}");
                let sbc = sbm.entry(sid.to_string()).or_insert_with_key(|_| 0);
                *sbc += 1;
                if *sbc > 1 {
                    println!(" XXX=== {sid}: {sbc}");
                }
                let Ok(buf) = std::fs::read(format!("{dnm}/{sid}.bin")) else {
                    nsn += 1;
                    //println!(" NO SUB {sid} {nsn}");
                    continue;
                };
                let (sub, _): (PeaSub, usize) =
                    bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();

                let mut fids: Vec<_> = sub.feedm.keys().collect();
                fids.sort();
                for fid in fids {
                    let Some(fd) = sub.feedm.get(fid) else {
                        nfn += 1;
                        //println!(" NO FEEDER {nfn}");
                        continue;
                    };
                    tn += fd.tranm.len();
                    for (_tid, trn) in fd.tranm.iter() {
                        mn += trn.mets.len();
                        //for met in &trn.mets {}
                    }
                }
            }
        }
    }
    println!(
        "pn:{pno} sn:{}  tn:{tn} mn:{mn}   NSN:{nsn} NFN:{nfn}    ctrs:{n_ctrs} cmts:{n_cmts} bils:{n_bils} m2bs:{n_m2bs}",
        sbm.len()
    );
    // pn:75 sn:721  tn:730068 mn:18143441   NSN:92 NFN:0    ctrs:785,655 cmts:21,580,648 bils:19,544,102 m2bs:21,580,648
    //pn:75 sn:721  tn:730,068 mn:18,143,441   NSN:92 NFN:0
    Ok(())
}
