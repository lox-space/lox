use crate::bodies::NaifId;

pub struct Moon;

impl NaifId for Moon {
    fn id() -> i32 {
        301
    }
}

pub struct Phobos;

impl NaifId for Phobos {
    fn id() -> i32 {
        401
    }
}

pub struct Deimos;

impl NaifId for Deimos {
    fn id() -> i32 {
        402
    }
}

// 599 Jupiter
//
// 501 Io          502 Europa      503 Ganymede    504 Callisto
// 505 Amalthea    506 Himalia     507 Elara       508 Pasiphae
// 509 Sinope      510 Lysithea    511 Carme       512 Ananke
// 513 Leda        514 Thebe       515 Adrastea    516 Metis
//
//
// 699 Saturn
//
// 601 Mimas       602 Enceladus   603 Tethys      604 Dione
// 605 Rhea        606 Titan       607 Hyperion    608 Iapetus
// 609 Phoebe      610 Janus       611 Epimetheus  612 Helene
// 613 Telesto     614 Calypso     615 Atlas       616 Prometheus
// 617 Pandora     618 Pan         632 Methone     633 Pallene
// 634 Polydeuces  635 Daphnis     649 Anthe       653 Aegaeon
//
//
// 799 Uranus
//
// 701 Ariel       702 Umbriel     703 Titania     704 Oberon
// 705 Miranda     706 Cordelia    707 Ophelia     708 Bianca
// 709 Cressida    710 Desdemona   711 Juliet      712 Portia
// 713 Rosalind    714 Belinda     715 Puck
//
//
// 899 Neptune
//
// 801 Triton      802 Nereid      803 Naiad       804 Thalassa
// 805 Despina     806 Galatea     807 Larissa     808 Proteus
//
//
// 999 Pluto
//
// 901 Charon
//
//
