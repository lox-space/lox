pub mod barycenters;
#[allow(clippy::approx_constant, clippy::excessive_precision)]
pub mod pck_constants;

// 1  Mercury barycenter
// 2  Venus barycenter
// 3  Earth barycenter
// 4  Mars barycenter
// 5  Jupiter barycenter
// 6  Saturn barycenter
// 7  Uranus barycenter
// 8  Neptune barycenter
// 9  Pluto barycenter
// 10 Sun
//
//
// 199 Mercury
//
//
// 299 Venus
//
//
// 399 Earth
//
// 301 Moon
//
//
// 499 Mars
//
// 401 Phobos      402 Deimos
//
//
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
// 1000005 Comet 19P/Borrelly
// 1000012 Comet 67P/Churyumov-Gerasimenko
// 1000036 Comet Halley
// 1000041 Comet Hartley 2
// 1000093 Comet 9P/Tempel 1
// 1000107 Comet 81P/Wild 2
//
// 2000001 Asteroid Ceres
// 2000002 Asteroid Pallas
// 2000016 Asteroid Psyche
// 2000004 Asteroid Vesta
// 2000021 Asteroid Lutetia
// 2000052 Asteroid 52 Europa
// 2000216 Asteroid Kleopatra
// 2000253 Asteroid Mathilde
// 2000433 Asteroid Eros
// 2000511 Asteroid Davida
// 2002867 Asteroid Steins
// 2004179 Asteroid Toutatis
// 2025143 Asteroid Itokawa
// 2431010 Asteroid Ida
// 9511010 Asteroid Gaspra

pub trait NaifId {
    fn id() -> i32;
}

pub struct Sun;

// 199 Mercury
//
//
// 299 Venus
//
//
// 399 Earth
//
// 301 Moon
//
//
// 499 Mars
//
// 401 Phobos      402 Deimos
//
//
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
// 1000005 Comet 19P/Borrelly
// 1000012 Comet 67P/Churyumov-Gerasimenko
// 1000036 Comet Halley
// 1000041 Comet Hartley 2
// 1000093 Comet 9P/Tempel 1
// 1000107 Comet 81P/Wild 2
//
// 2000001 Asteroid Ceres
// 2000002 Asteroid Pallas
// 2000016 Asteroid Psyche
// 2000004 Asteroid Vesta
// 2000021 Asteroid Lutetia
// 2000052 Asteroid 52 Europa
// 2000216 Asteroid Kleopatra
// 2000253 Asteroid Mathilde
// 2000433 Asteroid Eros
// 2000511 Asteroid Davida
// 2002867 Asteroid Steins
// 2004179 Asteroid Toutatis
// 2025143 Asteroid Itokawa
// 2431010 Asteroid Ida
// 9511010 Asteroid Gaspra
