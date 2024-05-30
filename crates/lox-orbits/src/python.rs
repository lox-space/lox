/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::frames::{Icrf, ReferenceFrame};
use crate::states::State;
use lox_bodies::python::PyBody;
use lox_time::python::time::PyTime;
use pyo3::pyclass;

#[pyclass]
pub enum PyFrame {
    Icrf,
    IauSun,
    IauMercury,
    IauVenus,
    IauEarth,
    IauMars,
    IauJupiter,
    IauSaturn,
    IauUranus,
    IauNeptune,
    IauPluto,
    IauMoon,
    IauPhobos,
    IauDeimos,
    IauIo,
    IauEuropa,
    IauGanymede,
    IauCallisto,
    IauAmalthea,
    IauHimalia,
    IauElara,
    IauPasiphae,
    IauSinope,
    IauLysithea,
    IauCarme,
    IauAnanke,
    IauLeda,
    IauThebe,
    IauAdrastea,
    IauMetis,
    IauCallirrhoe,
    IauThemisto,
    IauMagaclite,
    IauTaygete,
    IauChaldene,
    IauHarpalyke,
    IauKalyke,
    IauIocaste,
    IauErinome,
    IauIsonoe,
    IauPraxidike,
    IauAutonoe,
    IauThyone,
    IauHermippe,
    IauAitne,
    IauEurydome,
    IauEuanthe,
    IauEuporie,
    IauOrthosie,
    IauSponde,
    IauKale,
    IauPasithee,
    IauHegemone,
    IauMneme,
    IauAoede,
    IauThelxinoe,
    IauArche,
    IauKallichore,
    IauHelike,
    IauCarpo,
    IauEukelade,
    IauCyllene,
    IauKore,
    IauHerse,
    IauDia,
    IauMimas,
    IauEnceladus,
    IauTethys,
    IauDione,
    IauRhea,
    IauTitan,
    IauHyperion,
    IauIapetus,
    IauPhoebe,
    IauJanus,
    IauEpimetheus,
    IauHelene,
    IauTelesto,
    IauCalypso,
    IauAtlas,
    IauPrometheus,
    IauPandora,
    IauPan,
    IauYmir,
    IauPaaliaq,
    IauTarvos,
    IauIjiraq,
    IauSuttungr,
    IauKiviuq,
    IauMundilfari,
    IauAlbiorix,
    IauSkathi,
    IauErriapus,
    IauSiarnaq,
    IauThrymr,
    IauNarvi,
    IauMethone,
    IauPallene,
    IauPolydeuces,
    IauDaphnis,
    IauAegir,
    IauBebhionn,
    IauBergelmir,
    IauBestla,
    IauFarbauti,
    IauFenrir,
    IauFornjot,
    IauHati,
    IauHyrrokkin,
    IauKari,
    IauLoge,
    IauSkoll,
    IauSurtur,
    IauAnthe,
    IauJarnsaxa,
    IauGreip,
    IauTarqeq,
    IauAegaeon,
    IauAriel,
    IauUmbriel,
    IauTitania,
    IauOberon,
    IauMiranda,
    IauCordelia,
    IauOphelia,
    IauBianca,
    IauCressida,
    IauDesdemona,
    IauJuliet,
    IauPortia,
    IauRosalind,
    IauBelinda,
    IauPuck,
    IauCaliban,
    IauSycorax,
    IauProspero,
    IauSetebos,
    IauStephano,
    IauTrinculo,
    IauFrancisco,
    IauMargaret,
    IauFerdinand,
    IauPerdita,
    IauMab,
    IauCupid,
    IauTriton,
    IauNereid,
    IauNaiad,
    IauThalassa,
    IauDespina,
    IauGalatea,
    IauLarissa,
    IauProteus,
    IauHalimede,
    IauPsamathe,
    IauSao,
    IauLaomedeia,
    IauNeso,
    IauCharon,
    IauNix,
    IauHydra,
    IauKerberos,
    IauStyx,
    IauGaspra,
    IauIda,
    IauDactyl,
    IauCeres,
    IauPallas,
    IauVesta,
    IauPsyche,
    IauLutetia,
    IauKleopatra,
    IauEros,
    IauDavida,
    IauMathilde,
    IauSteins,
    IauBraille,
    IauWilsonHarrington,
    IauToutatis,
    IauItokawa,
    IauBennu,
}

impl ReferenceFrame for PyFrame {
    fn name(&self) -> &str {
        match self {
            PyFrame::Icrf => {}
            PyFrame::IauSun => {}
            PyFrame::IauMercury => {}
            PyFrame::IauVenus => {}
            PyFrame::IauEarth => {}
            PyFrame::IauMars => {}
            PyFrame::IauJupiter => {}
            PyFrame::IauSaturn => {}
            PyFrame::IauUranus => {}
            PyFrame::IauNeptune => {}
            PyFrame::IauPluto => {}
            PyFrame::IauMoon => {}
            PyFrame::IauPhobos => {}
            PyFrame::IauDeimos => {}
            PyFrame::IauIo => {}
            PyFrame::IauEuropa => {}
            PyFrame::IauGanymede => {}
            PyFrame::IauCallisto => {}
            PyFrame::IauAmalthea => {}
            PyFrame::IauHimalia => {}
            PyFrame::IauElara => {}
            PyFrame::IauPasiphae => {}
            PyFrame::IauSinope => {}
            PyFrame::IauLysithea => {}
            PyFrame::IauCarme => {}
            PyFrame::IauAnanke => {}
            PyFrame::IauLeda => {}
            PyFrame::IauThebe => {}
            PyFrame::IauAdrastea => {}
            PyFrame::IauMetis => {}
            PyFrame::IauCallirrhoe => {}
            PyFrame::IauThemisto => {}
            PyFrame::IauMagaclite => {}
            PyFrame::IauTaygete => {}
            PyFrame::IauChaldene => {}
            PyFrame::IauHarpalyke => {}
            PyFrame::IauKalyke => {}
            PyFrame::IauIocaste => {}
            PyFrame::IauErinome => {}
            PyFrame::IauIsonoe => {}
            PyFrame::IauPraxidike => {}
            PyFrame::IauAutonoe => {}
            PyFrame::IauThyone => {}
            PyFrame::IauHermippe => {}
            PyFrame::IauAitne => {}
            PyFrame::IauEurydome => {}
            PyFrame::IauEuanthe => {}
            PyFrame::IauEuporie => {}
            PyFrame::IauOrthosie => {}
            PyFrame::IauSponde => {}
            PyFrame::IauKale => {}
            PyFrame::IauPasithee => {}
            PyFrame::IauHegemone => {}
            PyFrame::IauMneme => {}
            PyFrame::IauAoede => {}
            PyFrame::IauThelxinoe => {}
            PyFrame::IauArche => {}
            PyFrame::IauKallichore => {}
            PyFrame::IauHelike => {}
            PyFrame::IauCarpo => {}
            PyFrame::IauEukelade => {}
            PyFrame::IauCyllene => {}
            PyFrame::IauKore => {}
            PyFrame::IauHerse => {}
            PyFrame::IauDia => {}
            PyFrame::IauMimas => {}
            PyFrame::IauEnceladus => {}
            PyFrame::IauTethys => {}
            PyFrame::IauDione => {}
            PyFrame::IauRhea => {}
            PyFrame::IauTitan => {}
            PyFrame::IauHyperion => {}
            PyFrame::IauIapetus => {}
            PyFrame::IauPhoebe => {}
            PyFrame::IauJanus => {}
            PyFrame::IauEpimetheus => {}
            PyFrame::IauHelene => {}
            PyFrame::IauTelesto => {}
            PyFrame::IauCalypso => {}
            PyFrame::IauAtlas => {}
            PyFrame::IauPrometheus => {}
            PyFrame::IauPandora => {}
            PyFrame::IauPan => {}
            PyFrame::IauYmir => {}
            PyFrame::IauPaaliaq => {}
            PyFrame::IauTarvos => {}
            PyFrame::IauIjiraq => {}
            PyFrame::IauSuttungr => {}
            PyFrame::IauKiviuq => {}
            PyFrame::IauMundilfari => {}
            PyFrame::IauAlbiorix => {}
            PyFrame::IauSkathi => {}
            PyFrame::IauErriapus => {}
            PyFrame::IauSiarnaq => {}
            PyFrame::IauThrymr => {}
            PyFrame::IauNarvi => {}
            PyFrame::IauMethone => {}
            PyFrame::IauPallene => {}
            PyFrame::IauPolydeuces => {}
            PyFrame::IauDaphnis => {}
            PyFrame::IauAegir => {}
            PyFrame::IauBebhionn => {}
            PyFrame::IauBergelmir => {}
            PyFrame::IauBestla => {}
            PyFrame::IauFarbauti => {}
            PyFrame::IauFenrir => {}
            PyFrame::IauFornjot => {}
            PyFrame::IauHati => {}
            PyFrame::IauHyrrokkin => {}
            PyFrame::IauKari => {}
            PyFrame::IauLoge => {}
            PyFrame::IauSkoll => {}
            PyFrame::IauSurtur => {}
            PyFrame::IauAnthe => {}
            PyFrame::IauJarnsaxa => {}
            PyFrame::IauGreip => {}
            PyFrame::IauTarqeq => {}
            PyFrame::IauAegaeon => {}
            PyFrame::IauAriel => {}
            PyFrame::IauUmbriel => {}
            PyFrame::IauTitania => {}
            PyFrame::IauOberon => {}
            PyFrame::IauMiranda => {}
            PyFrame::IauCordelia => {}
            PyFrame::IauOphelia => {}
            PyFrame::IauBianca => {}
            PyFrame::IauCressida => {}
            PyFrame::IauDesdemona => {}
            PyFrame::IauJuliet => {}
            PyFrame::IauPortia => {}
            PyFrame::IauRosalind => {}
            PyFrame::IauBelinda => {}
            PyFrame::IauPuck => {}
            PyFrame::IauCaliban => {}
            PyFrame::IauSycorax => {}
            PyFrame::IauProspero => {}
            PyFrame::IauSetebos => {}
            PyFrame::IauStephano => {}
            PyFrame::IauTrinculo => {}
            PyFrame::IauFrancisco => {}
            PyFrame::IauMargaret => {}
            PyFrame::IauFerdinand => {}
            PyFrame::IauPerdita => {}
            PyFrame::IauMab => {}
            PyFrame::IauCupid => {}
            PyFrame::IauTriton => {}
            PyFrame::IauNereid => {}
            PyFrame::IauNaiad => {}
            PyFrame::IauThalassa => {}
            PyFrame::IauDespina => {}
            PyFrame::IauGalatea => {}
            PyFrame::IauLarissa => {}
            PyFrame::IauProteus => {}
            PyFrame::IauHalimede => {}
            PyFrame::IauPsamathe => {}
            PyFrame::IauSao => {}
            PyFrame::IauLaomedeia => {}
            PyFrame::IauNeso => {}
            PyFrame::IauCharon => {}
            PyFrame::IauNix => {}
            PyFrame::IauHydra => {}
            PyFrame::IauKerberos => {}
            PyFrame::IauStyx => {}
            PyFrame::IauGaspra => {}
            PyFrame::IauIda => {}
            PyFrame::IauDactyl => {}
            PyFrame::IauCeres => {}
            PyFrame::IauPallas => {}
            PyFrame::IauVesta => {}
            PyFrame::IauPsyche => {}
            PyFrame::IauLutetia => {}
            PyFrame::IauKleopatra => {}
            PyFrame::IauEros => {}
            PyFrame::IauDavida => {}
            PyFrame::IauMathilde => {}
            PyFrame::IauSteins => {}
            PyFrame::IauBraille => {}
            PyFrame::IauWilsonHarrington => {}
            PyFrame::IauToutatis => {}
            PyFrame::IauItokawa => {}
            PyFrame::IauBennu => {}
        }
    }

    fn abbreviation(&self) -> &str {
        todo!()
    }
}

#[pyclass]
pub enum PyBodyfixed {
    Foo,
    Bar,
}

impl ReferenceFrame for PyBodyfixed {
    fn name(&self) -> &str {
        todo!()
    }

    fn abbreviation(&self) -> &str {
        todo!()
    }
}

#[pyclass]
pub struct IcrfState(State<PyTime, PyBody, Icrf>);

#[pyclass]
pub struct BodyfixedState(State<PyTime, PyBody, PyBodyfixed>);
