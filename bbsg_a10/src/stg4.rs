use crate::dcl::VarType;
use crate::dcl::{PeaAssVar, DNM};
use crate::stg2::ass_calc;
use crate::stg3::AojInfo;
use crate::utl::z2o;
use std::collections::HashMap;
use std::error::Error;

pub fn stage_04() -> Result<(), Box<dyn Error>> {
    let buf = std::fs::read(format!("{DNM}/000-aojm.bin")).unwrap();
    let Ok((aojm, _)): Result<(HashMap<String, AojInfo>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("000-AOJM not found".into());
    };
    for (i, (k, aoj)) in aojm.iter().enumerate() {
        println!("{i}.{k} {:?}", aoj.sbids);
        let mut v_tras = Vec::<PeaAssVar>::new();
        for sid in aoj.sbids.iter() {
            let Ok(buf) = std::fs::read(format!("{DNM}/{sid}-rw4.bin")) else {
                continue;
            };
            let (mut v_tras_raw, _): (Vec<PeaAssVar>, usize) =
                bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
            if v_tras_raw.is_empty() {
                println!("    {sid} - NO data ");
                continue;
            }
            for tras in v_tras_raw.iter_mut() {
                //println!("       {} = {k}", tras.aojcd);
                if tras.aojcd == *k {
                    v_tras.push(tras.clone());
                    //
                    // re-calculation of value
                    tras.v[VarType::LvPowSatTr as usize].v = tras.v[VarType::PkPowTr as usize].v
                        / z2o(tras.v[VarType::PwCapTr as usize].v);
                    tras.v[VarType::CntLvPowSatTr as usize].v =
                        if tras.v[VarType::LvPowSatTr as usize].v > 0.8f32 {
                            1f32
                        } else {
                            0f32
                        };
                    tras.v[VarType::ChgStnCap as usize].v = tras.v[VarType::ChgStnCapTr as usize].v;
                    tras.v[VarType::ChgStnSell as usize].v =
                        tras.v[VarType::ChgStnSellTr as usize].v;
                    tras.v[VarType::MvPowSatTr as usize].v = tras.v[VarType::MaxPosPowSub as usize]
                        .v
                        / z2o(tras.v[VarType::SubPowCap as usize].v);
                    tras.v[VarType::MvVspp as usize].v = tras.v[VarType::VsppMv as usize].v;
                    tras.v[VarType::HvSpp as usize].v = tras.v[VarType::SppHv as usize].v;
                    tras.v[VarType::SmallSell as usize].v = tras.v[VarType::SmallSellTr as usize].v;
                    tras.v[VarType::LargeSell as usize].v = tras.v[VarType::LargeSellTr as usize].v;
                    tras.v[VarType::UnbalPow as usize].v = tras.v[VarType::UnbalPowTr as usize].v;
                    let v = tras.v[VarType::UnbalPowTr as usize].v
                        / z2o(tras.v[VarType::PwCapTr as usize].v);
                    tras.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
                    // end of recalculation

                    ass_calc(tras)?;
                }
            }
        }
        //////////////////////////////////////////////
        //  raking for AOJs
        //let mut v_aojas: Vec<_> = m_aojas.into_iter().map(|(_, v)| v).collect();

        for aojas in v_tras.iter_mut() {
            let fir_cpx_opx = aojas.vy[VarType::FirCstRate.tousz()].clone();
            let guess = Some(0.);
            let fir: Vec<f64> = fir_cpx_opx.iter().map(|n| *n as f64).collect();
            let s0 = fir.iter().sum::<f64>();
            let firr = if s0 > 0f64 {
                financial::irr(&fir, guess).unwrap_or(0f64)
            } else {
                0f64
            };
            aojas.v[VarType::FirCstRate.tousz()].v = firr as f32;
        }

        let mut uc1_v: Vec<_> = v_tras
            .iter()
            .enumerate()
            .map(|(i, s)| (s.v[VarType::Uc1Val as usize].v, i))
            .collect();
        uc1_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (r, (_, i)) in uc1_v.iter().enumerate() {
            v_tras[*i].v[VarType::Uc1Rank as usize].v = r as f32 + 1.0;
        }
        let mut uc2_v: Vec<_> = v_tras
            .iter()
            .enumerate()
            .map(|(i, s)| (s.v[VarType::Uc2Val as usize].v, i))
            .collect();
        uc2_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (r, (_, i)) in uc2_v.iter().enumerate() {
            v_tras[*i].v[VarType::Uc2Rank as usize].v = r as f32 + 1.0;
        }
        let mut uc3_v: Vec<_> = v_tras
            .iter()
            .enumerate()
            .map(|(i, s)| (s.v[VarType::Uc3Val as usize].v, i))
            .collect();
        uc3_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (r, (_, i)) in uc3_v.iter().enumerate() {
            v_tras[*i].v[VarType::Uc3Rank as usize].v = r as f32 + 1.0;
        }

        let bin: Vec<u8> = bincode::encode_to_vec(&v_tras, bincode::config::standard()).unwrap();
        let fnm = format!("{DNM}/AOJ-{k}-assrw.bin");
        println!(" AOJ ASS = {fnm} {} added", v_tras.len());
        std::fs::write(fnm, bin).unwrap();
    }

    Ok(())
}
