pub use impl_bitfield::BitfieldSpecifier;
pub use impl_bitfield::bitfield;

// In the shared configuration crate

pub trait Specifier {
    const BITS: usize;
    type AssocType;
}

// pub use checks::MyType;

pub type MyType<T> = <<T as AnotherTrait>::BBB as TotalSizeIsMultipleOfEightBits>::AAA;

// pub type MyType2<T> = <T as DiscriminantInRange>::CCC;
pub type MyType2<T> = <<T as MyTempTrait>::CCC as DiscriminantInRange>::PlaceHolder;
// new here
pub trait DiscriminantInRange {
    type PlaceHolder;
}

pub enum True {}
pub enum False {}

pub trait MyTempTrait {
    type CCC;
}

impl MyTempTrait for [(); 0] {
    type CCC = True;
}

impl MyTempTrait for [(); 1] {
    type CCC = False;
}

impl DiscriminantInRange for True {
    type PlaceHolder = ();
}

// new here

pub trait AnotherTrait {
    type BBB;
}

impl AnotherTrait for [(); 0] {
    type BBB = ZeroMod8;
}

impl AnotherTrait for [(); 1] {
    type BBB = OneMod8;
}

impl AnotherTrait for [(); 2] {
    type BBB = TwoMod8;
}

impl AnotherTrait for [(); 3] {
    type BBB = ThreeMod8;
}

impl AnotherTrait for [(); 4] {
    type BBB = FourMod8;
}

impl AnotherTrait for [(); 5] {
    type BBB = FiveMod8;
}

impl AnotherTrait for [(); 6] {
    type BBB = SixMod8;
}

impl AnotherTrait for [(); 7] {
    type BBB = SevenMod8;
}
pub enum ZeroMod8 {}
pub enum OneMod8 {}

pub enum TwoMod8 {}
pub enum ThreeMod8 {}

pub enum FourMod8 {}
pub enum FiveMod8 {}

pub enum SixMod8 {}

pub enum SevenMod8 {}

pub trait TotalSizeIsMultipleOfEightBits {
    type AAA;
}

impl TotalSizeIsMultipleOfEightBits for ZeroMod8 {
    type AAA = ();
}
