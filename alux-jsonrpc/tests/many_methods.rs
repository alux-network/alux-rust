//! One declaration states 500 methods, and stating them costs what stating one of them costs.
//!
//! A declaration read as a nested type costs the square of its method count, or worse: at twenty
//! methods this file needed tens of gigabytes to check, and was killed before it finished. The
//! declaration is compiled to one registration per method instead, so what this guards is that
//! checking it stays ordinary work. The assertion is the compile; the test body only names the
//! program the declaration denotes.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_jsonrpc::{JsonRpcApiAlg, jsonrpc};
use core::future::Future;

/// Reads whatever the domain holds.
pub trait ReadingAlg {
    /// The value one reading denotes.
    type Reading;

    /// Returns the reading at `value`.
    fn read(&self, value: u32) -> impl Future<Output = Self::Reading> + Send;
}

/// Derives one operation per reading.
#[ext(name = ReadOperationExt, defunc)]
pub impl<This> This
where
    This: ReadingAlg,
{
    /// Reads the value at 0.
    async fn jsonrpc_read_0(&self, value: u32) -> This::Reading {
        self.read(value).await
    }

    /// Reads the value at 1.
    async fn jsonrpc_read_1(&self, value: u32) -> This::Reading {
        self.read(value + 1).await
    }

    /// Reads the value at 2.
    async fn jsonrpc_read_2(&self, value: u32) -> This::Reading {
        self.read(value + 2).await
    }

    /// Reads the value at 3.
    async fn jsonrpc_read_3(&self, value: u32) -> This::Reading {
        self.read(value + 3).await
    }

    /// Reads the value at 4.
    async fn jsonrpc_read_4(&self, value: u32) -> This::Reading {
        self.read(value + 4).await
    }

    /// Reads the value at 5.
    async fn jsonrpc_read_5(&self, value: u32) -> This::Reading {
        self.read(value + 5).await
    }

    /// Reads the value at 6.
    async fn jsonrpc_read_6(&self, value: u32) -> This::Reading {
        self.read(value + 6).await
    }

    /// Reads the value at 7.
    async fn jsonrpc_read_7(&self, value: u32) -> This::Reading {
        self.read(value + 7).await
    }

    /// Reads the value at 8.
    async fn jsonrpc_read_8(&self, value: u32) -> This::Reading {
        self.read(value + 8).await
    }

    /// Reads the value at 9.
    async fn jsonrpc_read_9(&self, value: u32) -> This::Reading {
        self.read(value + 9).await
    }

    /// Reads the value at 10.
    async fn jsonrpc_read_10(&self, value: u32) -> This::Reading {
        self.read(value + 10).await
    }

    /// Reads the value at 11.
    async fn jsonrpc_read_11(&self, value: u32) -> This::Reading {
        self.read(value + 11).await
    }

    /// Reads the value at 12.
    async fn jsonrpc_read_12(&self, value: u32) -> This::Reading {
        self.read(value + 12).await
    }

    /// Reads the value at 13.
    async fn jsonrpc_read_13(&self, value: u32) -> This::Reading {
        self.read(value + 13).await
    }

    /// Reads the value at 14.
    async fn jsonrpc_read_14(&self, value: u32) -> This::Reading {
        self.read(value + 14).await
    }

    /// Reads the value at 15.
    async fn jsonrpc_read_15(&self, value: u32) -> This::Reading {
        self.read(value + 15).await
    }

    /// Reads the value at 16.
    async fn jsonrpc_read_16(&self, value: u32) -> This::Reading {
        self.read(value + 16).await
    }

    /// Reads the value at 17.
    async fn jsonrpc_read_17(&self, value: u32) -> This::Reading {
        self.read(value + 17).await
    }

    /// Reads the value at 18.
    async fn jsonrpc_read_18(&self, value: u32) -> This::Reading {
        self.read(value + 18).await
    }

    /// Reads the value at 19.
    async fn jsonrpc_read_19(&self, value: u32) -> This::Reading {
        self.read(value + 19).await
    }

    /// Reads the value at 20.
    async fn jsonrpc_read_20(&self, value: u32) -> This::Reading {
        self.read(value + 20).await
    }

    /// Reads the value at 21.
    async fn jsonrpc_read_21(&self, value: u32) -> This::Reading {
        self.read(value + 21).await
    }

    /// Reads the value at 22.
    async fn jsonrpc_read_22(&self, value: u32) -> This::Reading {
        self.read(value + 22).await
    }

    /// Reads the value at 23.
    async fn jsonrpc_read_23(&self, value: u32) -> This::Reading {
        self.read(value + 23).await
    }

    /// Reads the value at 24.
    async fn jsonrpc_read_24(&self, value: u32) -> This::Reading {
        self.read(value + 24).await
    }

    /// Reads the value at 25.
    async fn jsonrpc_read_25(&self, value: u32) -> This::Reading {
        self.read(value + 25).await
    }

    /// Reads the value at 26.
    async fn jsonrpc_read_26(&self, value: u32) -> This::Reading {
        self.read(value + 26).await
    }

    /// Reads the value at 27.
    async fn jsonrpc_read_27(&self, value: u32) -> This::Reading {
        self.read(value + 27).await
    }

    /// Reads the value at 28.
    async fn jsonrpc_read_28(&self, value: u32) -> This::Reading {
        self.read(value + 28).await
    }

    /// Reads the value at 29.
    async fn jsonrpc_read_29(&self, value: u32) -> This::Reading {
        self.read(value + 29).await
    }

    /// Reads the value at 30.
    async fn jsonrpc_read_30(&self, value: u32) -> This::Reading {
        self.read(value + 30).await
    }

    /// Reads the value at 31.
    async fn jsonrpc_read_31(&self, value: u32) -> This::Reading {
        self.read(value + 31).await
    }

    /// Reads the value at 32.
    async fn jsonrpc_read_32(&self, value: u32) -> This::Reading {
        self.read(value + 32).await
    }

    /// Reads the value at 33.
    async fn jsonrpc_read_33(&self, value: u32) -> This::Reading {
        self.read(value + 33).await
    }

    /// Reads the value at 34.
    async fn jsonrpc_read_34(&self, value: u32) -> This::Reading {
        self.read(value + 34).await
    }

    /// Reads the value at 35.
    async fn jsonrpc_read_35(&self, value: u32) -> This::Reading {
        self.read(value + 35).await
    }

    /// Reads the value at 36.
    async fn jsonrpc_read_36(&self, value: u32) -> This::Reading {
        self.read(value + 36).await
    }

    /// Reads the value at 37.
    async fn jsonrpc_read_37(&self, value: u32) -> This::Reading {
        self.read(value + 37).await
    }

    /// Reads the value at 38.
    async fn jsonrpc_read_38(&self, value: u32) -> This::Reading {
        self.read(value + 38).await
    }

    /// Reads the value at 39.
    async fn jsonrpc_read_39(&self, value: u32) -> This::Reading {
        self.read(value + 39).await
    }

    /// Reads the value at 40.
    async fn jsonrpc_read_40(&self, value: u32) -> This::Reading {
        self.read(value + 40).await
    }

    /// Reads the value at 41.
    async fn jsonrpc_read_41(&self, value: u32) -> This::Reading {
        self.read(value + 41).await
    }

    /// Reads the value at 42.
    async fn jsonrpc_read_42(&self, value: u32) -> This::Reading {
        self.read(value + 42).await
    }

    /// Reads the value at 43.
    async fn jsonrpc_read_43(&self, value: u32) -> This::Reading {
        self.read(value + 43).await
    }

    /// Reads the value at 44.
    async fn jsonrpc_read_44(&self, value: u32) -> This::Reading {
        self.read(value + 44).await
    }

    /// Reads the value at 45.
    async fn jsonrpc_read_45(&self, value: u32) -> This::Reading {
        self.read(value + 45).await
    }

    /// Reads the value at 46.
    async fn jsonrpc_read_46(&self, value: u32) -> This::Reading {
        self.read(value + 46).await
    }

    /// Reads the value at 47.
    async fn jsonrpc_read_47(&self, value: u32) -> This::Reading {
        self.read(value + 47).await
    }

    /// Reads the value at 48.
    async fn jsonrpc_read_48(&self, value: u32) -> This::Reading {
        self.read(value + 48).await
    }

    /// Reads the value at 49.
    async fn jsonrpc_read_49(&self, value: u32) -> This::Reading {
        self.read(value + 49).await
    }

    /// Reads the value at 50.
    async fn jsonrpc_read_50(&self, value: u32) -> This::Reading {
        self.read(value + 50).await
    }

    /// Reads the value at 51.
    async fn jsonrpc_read_51(&self, value: u32) -> This::Reading {
        self.read(value + 51).await
    }

    /// Reads the value at 52.
    async fn jsonrpc_read_52(&self, value: u32) -> This::Reading {
        self.read(value + 52).await
    }

    /// Reads the value at 53.
    async fn jsonrpc_read_53(&self, value: u32) -> This::Reading {
        self.read(value + 53).await
    }

    /// Reads the value at 54.
    async fn jsonrpc_read_54(&self, value: u32) -> This::Reading {
        self.read(value + 54).await
    }

    /// Reads the value at 55.
    async fn jsonrpc_read_55(&self, value: u32) -> This::Reading {
        self.read(value + 55).await
    }

    /// Reads the value at 56.
    async fn jsonrpc_read_56(&self, value: u32) -> This::Reading {
        self.read(value + 56).await
    }

    /// Reads the value at 57.
    async fn jsonrpc_read_57(&self, value: u32) -> This::Reading {
        self.read(value + 57).await
    }

    /// Reads the value at 58.
    async fn jsonrpc_read_58(&self, value: u32) -> This::Reading {
        self.read(value + 58).await
    }

    /// Reads the value at 59.
    async fn jsonrpc_read_59(&self, value: u32) -> This::Reading {
        self.read(value + 59).await
    }

    /// Reads the value at 60.
    async fn jsonrpc_read_60(&self, value: u32) -> This::Reading {
        self.read(value + 60).await
    }

    /// Reads the value at 61.
    async fn jsonrpc_read_61(&self, value: u32) -> This::Reading {
        self.read(value + 61).await
    }

    /// Reads the value at 62.
    async fn jsonrpc_read_62(&self, value: u32) -> This::Reading {
        self.read(value + 62).await
    }

    /// Reads the value at 63.
    async fn jsonrpc_read_63(&self, value: u32) -> This::Reading {
        self.read(value + 63).await
    }

    /// Reads the value at 64.
    async fn jsonrpc_read_64(&self, value: u32) -> This::Reading {
        self.read(value + 64).await
    }

    /// Reads the value at 65.
    async fn jsonrpc_read_65(&self, value: u32) -> This::Reading {
        self.read(value + 65).await
    }

    /// Reads the value at 66.
    async fn jsonrpc_read_66(&self, value: u32) -> This::Reading {
        self.read(value + 66).await
    }

    /// Reads the value at 67.
    async fn jsonrpc_read_67(&self, value: u32) -> This::Reading {
        self.read(value + 67).await
    }

    /// Reads the value at 68.
    async fn jsonrpc_read_68(&self, value: u32) -> This::Reading {
        self.read(value + 68).await
    }

    /// Reads the value at 69.
    async fn jsonrpc_read_69(&self, value: u32) -> This::Reading {
        self.read(value + 69).await
    }

    /// Reads the value at 70.
    async fn jsonrpc_read_70(&self, value: u32) -> This::Reading {
        self.read(value + 70).await
    }

    /// Reads the value at 71.
    async fn jsonrpc_read_71(&self, value: u32) -> This::Reading {
        self.read(value + 71).await
    }

    /// Reads the value at 72.
    async fn jsonrpc_read_72(&self, value: u32) -> This::Reading {
        self.read(value + 72).await
    }

    /// Reads the value at 73.
    async fn jsonrpc_read_73(&self, value: u32) -> This::Reading {
        self.read(value + 73).await
    }

    /// Reads the value at 74.
    async fn jsonrpc_read_74(&self, value: u32) -> This::Reading {
        self.read(value + 74).await
    }

    /// Reads the value at 75.
    async fn jsonrpc_read_75(&self, value: u32) -> This::Reading {
        self.read(value + 75).await
    }

    /// Reads the value at 76.
    async fn jsonrpc_read_76(&self, value: u32) -> This::Reading {
        self.read(value + 76).await
    }

    /// Reads the value at 77.
    async fn jsonrpc_read_77(&self, value: u32) -> This::Reading {
        self.read(value + 77).await
    }

    /// Reads the value at 78.
    async fn jsonrpc_read_78(&self, value: u32) -> This::Reading {
        self.read(value + 78).await
    }

    /// Reads the value at 79.
    async fn jsonrpc_read_79(&self, value: u32) -> This::Reading {
        self.read(value + 79).await
    }

    /// Reads the value at 80.
    async fn jsonrpc_read_80(&self, value: u32) -> This::Reading {
        self.read(value + 80).await
    }

    /// Reads the value at 81.
    async fn jsonrpc_read_81(&self, value: u32) -> This::Reading {
        self.read(value + 81).await
    }

    /// Reads the value at 82.
    async fn jsonrpc_read_82(&self, value: u32) -> This::Reading {
        self.read(value + 82).await
    }

    /// Reads the value at 83.
    async fn jsonrpc_read_83(&self, value: u32) -> This::Reading {
        self.read(value + 83).await
    }

    /// Reads the value at 84.
    async fn jsonrpc_read_84(&self, value: u32) -> This::Reading {
        self.read(value + 84).await
    }

    /// Reads the value at 85.
    async fn jsonrpc_read_85(&self, value: u32) -> This::Reading {
        self.read(value + 85).await
    }

    /// Reads the value at 86.
    async fn jsonrpc_read_86(&self, value: u32) -> This::Reading {
        self.read(value + 86).await
    }

    /// Reads the value at 87.
    async fn jsonrpc_read_87(&self, value: u32) -> This::Reading {
        self.read(value + 87).await
    }

    /// Reads the value at 88.
    async fn jsonrpc_read_88(&self, value: u32) -> This::Reading {
        self.read(value + 88).await
    }

    /// Reads the value at 89.
    async fn jsonrpc_read_89(&self, value: u32) -> This::Reading {
        self.read(value + 89).await
    }

    /// Reads the value at 90.
    async fn jsonrpc_read_90(&self, value: u32) -> This::Reading {
        self.read(value + 90).await
    }

    /// Reads the value at 91.
    async fn jsonrpc_read_91(&self, value: u32) -> This::Reading {
        self.read(value + 91).await
    }

    /// Reads the value at 92.
    async fn jsonrpc_read_92(&self, value: u32) -> This::Reading {
        self.read(value + 92).await
    }

    /// Reads the value at 93.
    async fn jsonrpc_read_93(&self, value: u32) -> This::Reading {
        self.read(value + 93).await
    }

    /// Reads the value at 94.
    async fn jsonrpc_read_94(&self, value: u32) -> This::Reading {
        self.read(value + 94).await
    }

    /// Reads the value at 95.
    async fn jsonrpc_read_95(&self, value: u32) -> This::Reading {
        self.read(value + 95).await
    }

    /// Reads the value at 96.
    async fn jsonrpc_read_96(&self, value: u32) -> This::Reading {
        self.read(value + 96).await
    }

    /// Reads the value at 97.
    async fn jsonrpc_read_97(&self, value: u32) -> This::Reading {
        self.read(value + 97).await
    }

    /// Reads the value at 98.
    async fn jsonrpc_read_98(&self, value: u32) -> This::Reading {
        self.read(value + 98).await
    }

    /// Reads the value at 99.
    async fn jsonrpc_read_99(&self, value: u32) -> This::Reading {
        self.read(value + 99).await
    }

    /// Reads the value at 100.
    async fn jsonrpc_read_100(&self, value: u32) -> This::Reading {
        self.read(value + 100).await
    }

    /// Reads the value at 101.
    async fn jsonrpc_read_101(&self, value: u32) -> This::Reading {
        self.read(value + 101).await
    }

    /// Reads the value at 102.
    async fn jsonrpc_read_102(&self, value: u32) -> This::Reading {
        self.read(value + 102).await
    }

    /// Reads the value at 103.
    async fn jsonrpc_read_103(&self, value: u32) -> This::Reading {
        self.read(value + 103).await
    }

    /// Reads the value at 104.
    async fn jsonrpc_read_104(&self, value: u32) -> This::Reading {
        self.read(value + 104).await
    }

    /// Reads the value at 105.
    async fn jsonrpc_read_105(&self, value: u32) -> This::Reading {
        self.read(value + 105).await
    }

    /// Reads the value at 106.
    async fn jsonrpc_read_106(&self, value: u32) -> This::Reading {
        self.read(value + 106).await
    }

    /// Reads the value at 107.
    async fn jsonrpc_read_107(&self, value: u32) -> This::Reading {
        self.read(value + 107).await
    }

    /// Reads the value at 108.
    async fn jsonrpc_read_108(&self, value: u32) -> This::Reading {
        self.read(value + 108).await
    }

    /// Reads the value at 109.
    async fn jsonrpc_read_109(&self, value: u32) -> This::Reading {
        self.read(value + 109).await
    }

    /// Reads the value at 110.
    async fn jsonrpc_read_110(&self, value: u32) -> This::Reading {
        self.read(value + 110).await
    }

    /// Reads the value at 111.
    async fn jsonrpc_read_111(&self, value: u32) -> This::Reading {
        self.read(value + 111).await
    }

    /// Reads the value at 112.
    async fn jsonrpc_read_112(&self, value: u32) -> This::Reading {
        self.read(value + 112).await
    }

    /// Reads the value at 113.
    async fn jsonrpc_read_113(&self, value: u32) -> This::Reading {
        self.read(value + 113).await
    }

    /// Reads the value at 114.
    async fn jsonrpc_read_114(&self, value: u32) -> This::Reading {
        self.read(value + 114).await
    }

    /// Reads the value at 115.
    async fn jsonrpc_read_115(&self, value: u32) -> This::Reading {
        self.read(value + 115).await
    }

    /// Reads the value at 116.
    async fn jsonrpc_read_116(&self, value: u32) -> This::Reading {
        self.read(value + 116).await
    }

    /// Reads the value at 117.
    async fn jsonrpc_read_117(&self, value: u32) -> This::Reading {
        self.read(value + 117).await
    }

    /// Reads the value at 118.
    async fn jsonrpc_read_118(&self, value: u32) -> This::Reading {
        self.read(value + 118).await
    }

    /// Reads the value at 119.
    async fn jsonrpc_read_119(&self, value: u32) -> This::Reading {
        self.read(value + 119).await
    }

    /// Reads the value at 120.
    async fn jsonrpc_read_120(&self, value: u32) -> This::Reading {
        self.read(value + 120).await
    }

    /// Reads the value at 121.
    async fn jsonrpc_read_121(&self, value: u32) -> This::Reading {
        self.read(value + 121).await
    }

    /// Reads the value at 122.
    async fn jsonrpc_read_122(&self, value: u32) -> This::Reading {
        self.read(value + 122).await
    }

    /// Reads the value at 123.
    async fn jsonrpc_read_123(&self, value: u32) -> This::Reading {
        self.read(value + 123).await
    }

    /// Reads the value at 124.
    async fn jsonrpc_read_124(&self, value: u32) -> This::Reading {
        self.read(value + 124).await
    }

    /// Reads the value at 125.
    async fn jsonrpc_read_125(&self, value: u32) -> This::Reading {
        self.read(value + 125).await
    }

    /// Reads the value at 126.
    async fn jsonrpc_read_126(&self, value: u32) -> This::Reading {
        self.read(value + 126).await
    }

    /// Reads the value at 127.
    async fn jsonrpc_read_127(&self, value: u32) -> This::Reading {
        self.read(value + 127).await
    }

    /// Reads the value at 128.
    async fn jsonrpc_read_128(&self, value: u32) -> This::Reading {
        self.read(value + 128).await
    }

    /// Reads the value at 129.
    async fn jsonrpc_read_129(&self, value: u32) -> This::Reading {
        self.read(value + 129).await
    }

    /// Reads the value at 130.
    async fn jsonrpc_read_130(&self, value: u32) -> This::Reading {
        self.read(value + 130).await
    }

    /// Reads the value at 131.
    async fn jsonrpc_read_131(&self, value: u32) -> This::Reading {
        self.read(value + 131).await
    }

    /// Reads the value at 132.
    async fn jsonrpc_read_132(&self, value: u32) -> This::Reading {
        self.read(value + 132).await
    }

    /// Reads the value at 133.
    async fn jsonrpc_read_133(&self, value: u32) -> This::Reading {
        self.read(value + 133).await
    }

    /// Reads the value at 134.
    async fn jsonrpc_read_134(&self, value: u32) -> This::Reading {
        self.read(value + 134).await
    }

    /// Reads the value at 135.
    async fn jsonrpc_read_135(&self, value: u32) -> This::Reading {
        self.read(value + 135).await
    }

    /// Reads the value at 136.
    async fn jsonrpc_read_136(&self, value: u32) -> This::Reading {
        self.read(value + 136).await
    }

    /// Reads the value at 137.
    async fn jsonrpc_read_137(&self, value: u32) -> This::Reading {
        self.read(value + 137).await
    }

    /// Reads the value at 138.
    async fn jsonrpc_read_138(&self, value: u32) -> This::Reading {
        self.read(value + 138).await
    }

    /// Reads the value at 139.
    async fn jsonrpc_read_139(&self, value: u32) -> This::Reading {
        self.read(value + 139).await
    }

    /// Reads the value at 140.
    async fn jsonrpc_read_140(&self, value: u32) -> This::Reading {
        self.read(value + 140).await
    }

    /// Reads the value at 141.
    async fn jsonrpc_read_141(&self, value: u32) -> This::Reading {
        self.read(value + 141).await
    }

    /// Reads the value at 142.
    async fn jsonrpc_read_142(&self, value: u32) -> This::Reading {
        self.read(value + 142).await
    }

    /// Reads the value at 143.
    async fn jsonrpc_read_143(&self, value: u32) -> This::Reading {
        self.read(value + 143).await
    }

    /// Reads the value at 144.
    async fn jsonrpc_read_144(&self, value: u32) -> This::Reading {
        self.read(value + 144).await
    }

    /// Reads the value at 145.
    async fn jsonrpc_read_145(&self, value: u32) -> This::Reading {
        self.read(value + 145).await
    }

    /// Reads the value at 146.
    async fn jsonrpc_read_146(&self, value: u32) -> This::Reading {
        self.read(value + 146).await
    }

    /// Reads the value at 147.
    async fn jsonrpc_read_147(&self, value: u32) -> This::Reading {
        self.read(value + 147).await
    }

    /// Reads the value at 148.
    async fn jsonrpc_read_148(&self, value: u32) -> This::Reading {
        self.read(value + 148).await
    }

    /// Reads the value at 149.
    async fn jsonrpc_read_149(&self, value: u32) -> This::Reading {
        self.read(value + 149).await
    }

    /// Reads the value at 150.
    async fn jsonrpc_read_150(&self, value: u32) -> This::Reading {
        self.read(value + 150).await
    }

    /// Reads the value at 151.
    async fn jsonrpc_read_151(&self, value: u32) -> This::Reading {
        self.read(value + 151).await
    }

    /// Reads the value at 152.
    async fn jsonrpc_read_152(&self, value: u32) -> This::Reading {
        self.read(value + 152).await
    }

    /// Reads the value at 153.
    async fn jsonrpc_read_153(&self, value: u32) -> This::Reading {
        self.read(value + 153).await
    }

    /// Reads the value at 154.
    async fn jsonrpc_read_154(&self, value: u32) -> This::Reading {
        self.read(value + 154).await
    }

    /// Reads the value at 155.
    async fn jsonrpc_read_155(&self, value: u32) -> This::Reading {
        self.read(value + 155).await
    }

    /// Reads the value at 156.
    async fn jsonrpc_read_156(&self, value: u32) -> This::Reading {
        self.read(value + 156).await
    }

    /// Reads the value at 157.
    async fn jsonrpc_read_157(&self, value: u32) -> This::Reading {
        self.read(value + 157).await
    }

    /// Reads the value at 158.
    async fn jsonrpc_read_158(&self, value: u32) -> This::Reading {
        self.read(value + 158).await
    }

    /// Reads the value at 159.
    async fn jsonrpc_read_159(&self, value: u32) -> This::Reading {
        self.read(value + 159).await
    }

    /// Reads the value at 160.
    async fn jsonrpc_read_160(&self, value: u32) -> This::Reading {
        self.read(value + 160).await
    }

    /// Reads the value at 161.
    async fn jsonrpc_read_161(&self, value: u32) -> This::Reading {
        self.read(value + 161).await
    }

    /// Reads the value at 162.
    async fn jsonrpc_read_162(&self, value: u32) -> This::Reading {
        self.read(value + 162).await
    }

    /// Reads the value at 163.
    async fn jsonrpc_read_163(&self, value: u32) -> This::Reading {
        self.read(value + 163).await
    }

    /// Reads the value at 164.
    async fn jsonrpc_read_164(&self, value: u32) -> This::Reading {
        self.read(value + 164).await
    }

    /// Reads the value at 165.
    async fn jsonrpc_read_165(&self, value: u32) -> This::Reading {
        self.read(value + 165).await
    }

    /// Reads the value at 166.
    async fn jsonrpc_read_166(&self, value: u32) -> This::Reading {
        self.read(value + 166).await
    }

    /// Reads the value at 167.
    async fn jsonrpc_read_167(&self, value: u32) -> This::Reading {
        self.read(value + 167).await
    }

    /// Reads the value at 168.
    async fn jsonrpc_read_168(&self, value: u32) -> This::Reading {
        self.read(value + 168).await
    }

    /// Reads the value at 169.
    async fn jsonrpc_read_169(&self, value: u32) -> This::Reading {
        self.read(value + 169).await
    }

    /// Reads the value at 170.
    async fn jsonrpc_read_170(&self, value: u32) -> This::Reading {
        self.read(value + 170).await
    }

    /// Reads the value at 171.
    async fn jsonrpc_read_171(&self, value: u32) -> This::Reading {
        self.read(value + 171).await
    }

    /// Reads the value at 172.
    async fn jsonrpc_read_172(&self, value: u32) -> This::Reading {
        self.read(value + 172).await
    }

    /// Reads the value at 173.
    async fn jsonrpc_read_173(&self, value: u32) -> This::Reading {
        self.read(value + 173).await
    }

    /// Reads the value at 174.
    async fn jsonrpc_read_174(&self, value: u32) -> This::Reading {
        self.read(value + 174).await
    }

    /// Reads the value at 175.
    async fn jsonrpc_read_175(&self, value: u32) -> This::Reading {
        self.read(value + 175).await
    }

    /// Reads the value at 176.
    async fn jsonrpc_read_176(&self, value: u32) -> This::Reading {
        self.read(value + 176).await
    }

    /// Reads the value at 177.
    async fn jsonrpc_read_177(&self, value: u32) -> This::Reading {
        self.read(value + 177).await
    }

    /// Reads the value at 178.
    async fn jsonrpc_read_178(&self, value: u32) -> This::Reading {
        self.read(value + 178).await
    }

    /// Reads the value at 179.
    async fn jsonrpc_read_179(&self, value: u32) -> This::Reading {
        self.read(value + 179).await
    }

    /// Reads the value at 180.
    async fn jsonrpc_read_180(&self, value: u32) -> This::Reading {
        self.read(value + 180).await
    }

    /// Reads the value at 181.
    async fn jsonrpc_read_181(&self, value: u32) -> This::Reading {
        self.read(value + 181).await
    }

    /// Reads the value at 182.
    async fn jsonrpc_read_182(&self, value: u32) -> This::Reading {
        self.read(value + 182).await
    }

    /// Reads the value at 183.
    async fn jsonrpc_read_183(&self, value: u32) -> This::Reading {
        self.read(value + 183).await
    }

    /// Reads the value at 184.
    async fn jsonrpc_read_184(&self, value: u32) -> This::Reading {
        self.read(value + 184).await
    }

    /// Reads the value at 185.
    async fn jsonrpc_read_185(&self, value: u32) -> This::Reading {
        self.read(value + 185).await
    }

    /// Reads the value at 186.
    async fn jsonrpc_read_186(&self, value: u32) -> This::Reading {
        self.read(value + 186).await
    }

    /// Reads the value at 187.
    async fn jsonrpc_read_187(&self, value: u32) -> This::Reading {
        self.read(value + 187).await
    }

    /// Reads the value at 188.
    async fn jsonrpc_read_188(&self, value: u32) -> This::Reading {
        self.read(value + 188).await
    }

    /// Reads the value at 189.
    async fn jsonrpc_read_189(&self, value: u32) -> This::Reading {
        self.read(value + 189).await
    }

    /// Reads the value at 190.
    async fn jsonrpc_read_190(&self, value: u32) -> This::Reading {
        self.read(value + 190).await
    }

    /// Reads the value at 191.
    async fn jsonrpc_read_191(&self, value: u32) -> This::Reading {
        self.read(value + 191).await
    }

    /// Reads the value at 192.
    async fn jsonrpc_read_192(&self, value: u32) -> This::Reading {
        self.read(value + 192).await
    }

    /// Reads the value at 193.
    async fn jsonrpc_read_193(&self, value: u32) -> This::Reading {
        self.read(value + 193).await
    }

    /// Reads the value at 194.
    async fn jsonrpc_read_194(&self, value: u32) -> This::Reading {
        self.read(value + 194).await
    }

    /// Reads the value at 195.
    async fn jsonrpc_read_195(&self, value: u32) -> This::Reading {
        self.read(value + 195).await
    }

    /// Reads the value at 196.
    async fn jsonrpc_read_196(&self, value: u32) -> This::Reading {
        self.read(value + 196).await
    }

    /// Reads the value at 197.
    async fn jsonrpc_read_197(&self, value: u32) -> This::Reading {
        self.read(value + 197).await
    }

    /// Reads the value at 198.
    async fn jsonrpc_read_198(&self, value: u32) -> This::Reading {
        self.read(value + 198).await
    }

    /// Reads the value at 199.
    async fn jsonrpc_read_199(&self, value: u32) -> This::Reading {
        self.read(value + 199).await
    }

    /// Reads the value at 200.
    async fn jsonrpc_read_200(&self, value: u32) -> This::Reading {
        self.read(value + 200).await
    }

    /// Reads the value at 201.
    async fn jsonrpc_read_201(&self, value: u32) -> This::Reading {
        self.read(value + 201).await
    }

    /// Reads the value at 202.
    async fn jsonrpc_read_202(&self, value: u32) -> This::Reading {
        self.read(value + 202).await
    }

    /// Reads the value at 203.
    async fn jsonrpc_read_203(&self, value: u32) -> This::Reading {
        self.read(value + 203).await
    }

    /// Reads the value at 204.
    async fn jsonrpc_read_204(&self, value: u32) -> This::Reading {
        self.read(value + 204).await
    }

    /// Reads the value at 205.
    async fn jsonrpc_read_205(&self, value: u32) -> This::Reading {
        self.read(value + 205).await
    }

    /// Reads the value at 206.
    async fn jsonrpc_read_206(&self, value: u32) -> This::Reading {
        self.read(value + 206).await
    }

    /// Reads the value at 207.
    async fn jsonrpc_read_207(&self, value: u32) -> This::Reading {
        self.read(value + 207).await
    }

    /// Reads the value at 208.
    async fn jsonrpc_read_208(&self, value: u32) -> This::Reading {
        self.read(value + 208).await
    }

    /// Reads the value at 209.
    async fn jsonrpc_read_209(&self, value: u32) -> This::Reading {
        self.read(value + 209).await
    }

    /// Reads the value at 210.
    async fn jsonrpc_read_210(&self, value: u32) -> This::Reading {
        self.read(value + 210).await
    }

    /// Reads the value at 211.
    async fn jsonrpc_read_211(&self, value: u32) -> This::Reading {
        self.read(value + 211).await
    }

    /// Reads the value at 212.
    async fn jsonrpc_read_212(&self, value: u32) -> This::Reading {
        self.read(value + 212).await
    }

    /// Reads the value at 213.
    async fn jsonrpc_read_213(&self, value: u32) -> This::Reading {
        self.read(value + 213).await
    }

    /// Reads the value at 214.
    async fn jsonrpc_read_214(&self, value: u32) -> This::Reading {
        self.read(value + 214).await
    }

    /// Reads the value at 215.
    async fn jsonrpc_read_215(&self, value: u32) -> This::Reading {
        self.read(value + 215).await
    }

    /// Reads the value at 216.
    async fn jsonrpc_read_216(&self, value: u32) -> This::Reading {
        self.read(value + 216).await
    }

    /// Reads the value at 217.
    async fn jsonrpc_read_217(&self, value: u32) -> This::Reading {
        self.read(value + 217).await
    }

    /// Reads the value at 218.
    async fn jsonrpc_read_218(&self, value: u32) -> This::Reading {
        self.read(value + 218).await
    }

    /// Reads the value at 219.
    async fn jsonrpc_read_219(&self, value: u32) -> This::Reading {
        self.read(value + 219).await
    }

    /// Reads the value at 220.
    async fn jsonrpc_read_220(&self, value: u32) -> This::Reading {
        self.read(value + 220).await
    }

    /// Reads the value at 221.
    async fn jsonrpc_read_221(&self, value: u32) -> This::Reading {
        self.read(value + 221).await
    }

    /// Reads the value at 222.
    async fn jsonrpc_read_222(&self, value: u32) -> This::Reading {
        self.read(value + 222).await
    }

    /// Reads the value at 223.
    async fn jsonrpc_read_223(&self, value: u32) -> This::Reading {
        self.read(value + 223).await
    }

    /// Reads the value at 224.
    async fn jsonrpc_read_224(&self, value: u32) -> This::Reading {
        self.read(value + 224).await
    }

    /// Reads the value at 225.
    async fn jsonrpc_read_225(&self, value: u32) -> This::Reading {
        self.read(value + 225).await
    }

    /// Reads the value at 226.
    async fn jsonrpc_read_226(&self, value: u32) -> This::Reading {
        self.read(value + 226).await
    }

    /// Reads the value at 227.
    async fn jsonrpc_read_227(&self, value: u32) -> This::Reading {
        self.read(value + 227).await
    }

    /// Reads the value at 228.
    async fn jsonrpc_read_228(&self, value: u32) -> This::Reading {
        self.read(value + 228).await
    }

    /// Reads the value at 229.
    async fn jsonrpc_read_229(&self, value: u32) -> This::Reading {
        self.read(value + 229).await
    }

    /// Reads the value at 230.
    async fn jsonrpc_read_230(&self, value: u32) -> This::Reading {
        self.read(value + 230).await
    }

    /// Reads the value at 231.
    async fn jsonrpc_read_231(&self, value: u32) -> This::Reading {
        self.read(value + 231).await
    }

    /// Reads the value at 232.
    async fn jsonrpc_read_232(&self, value: u32) -> This::Reading {
        self.read(value + 232).await
    }

    /// Reads the value at 233.
    async fn jsonrpc_read_233(&self, value: u32) -> This::Reading {
        self.read(value + 233).await
    }

    /// Reads the value at 234.
    async fn jsonrpc_read_234(&self, value: u32) -> This::Reading {
        self.read(value + 234).await
    }

    /// Reads the value at 235.
    async fn jsonrpc_read_235(&self, value: u32) -> This::Reading {
        self.read(value + 235).await
    }

    /// Reads the value at 236.
    async fn jsonrpc_read_236(&self, value: u32) -> This::Reading {
        self.read(value + 236).await
    }

    /// Reads the value at 237.
    async fn jsonrpc_read_237(&self, value: u32) -> This::Reading {
        self.read(value + 237).await
    }

    /// Reads the value at 238.
    async fn jsonrpc_read_238(&self, value: u32) -> This::Reading {
        self.read(value + 238).await
    }

    /// Reads the value at 239.
    async fn jsonrpc_read_239(&self, value: u32) -> This::Reading {
        self.read(value + 239).await
    }

    /// Reads the value at 240.
    async fn jsonrpc_read_240(&self, value: u32) -> This::Reading {
        self.read(value + 240).await
    }

    /// Reads the value at 241.
    async fn jsonrpc_read_241(&self, value: u32) -> This::Reading {
        self.read(value + 241).await
    }

    /// Reads the value at 242.
    async fn jsonrpc_read_242(&self, value: u32) -> This::Reading {
        self.read(value + 242).await
    }

    /// Reads the value at 243.
    async fn jsonrpc_read_243(&self, value: u32) -> This::Reading {
        self.read(value + 243).await
    }

    /// Reads the value at 244.
    async fn jsonrpc_read_244(&self, value: u32) -> This::Reading {
        self.read(value + 244).await
    }

    /// Reads the value at 245.
    async fn jsonrpc_read_245(&self, value: u32) -> This::Reading {
        self.read(value + 245).await
    }

    /// Reads the value at 246.
    async fn jsonrpc_read_246(&self, value: u32) -> This::Reading {
        self.read(value + 246).await
    }

    /// Reads the value at 247.
    async fn jsonrpc_read_247(&self, value: u32) -> This::Reading {
        self.read(value + 247).await
    }

    /// Reads the value at 248.
    async fn jsonrpc_read_248(&self, value: u32) -> This::Reading {
        self.read(value + 248).await
    }

    /// Reads the value at 249.
    async fn jsonrpc_read_249(&self, value: u32) -> This::Reading {
        self.read(value + 249).await
    }

    /// Reads the value at 250.
    async fn jsonrpc_read_250(&self, value: u32) -> This::Reading {
        self.read(value + 250).await
    }

    /// Reads the value at 251.
    async fn jsonrpc_read_251(&self, value: u32) -> This::Reading {
        self.read(value + 251).await
    }

    /// Reads the value at 252.
    async fn jsonrpc_read_252(&self, value: u32) -> This::Reading {
        self.read(value + 252).await
    }

    /// Reads the value at 253.
    async fn jsonrpc_read_253(&self, value: u32) -> This::Reading {
        self.read(value + 253).await
    }

    /// Reads the value at 254.
    async fn jsonrpc_read_254(&self, value: u32) -> This::Reading {
        self.read(value + 254).await
    }

    /// Reads the value at 255.
    async fn jsonrpc_read_255(&self, value: u32) -> This::Reading {
        self.read(value + 255).await
    }

    /// Reads the value at 256.
    async fn jsonrpc_read_256(&self, value: u32) -> This::Reading {
        self.read(value + 256).await
    }

    /// Reads the value at 257.
    async fn jsonrpc_read_257(&self, value: u32) -> This::Reading {
        self.read(value + 257).await
    }

    /// Reads the value at 258.
    async fn jsonrpc_read_258(&self, value: u32) -> This::Reading {
        self.read(value + 258).await
    }

    /// Reads the value at 259.
    async fn jsonrpc_read_259(&self, value: u32) -> This::Reading {
        self.read(value + 259).await
    }

    /// Reads the value at 260.
    async fn jsonrpc_read_260(&self, value: u32) -> This::Reading {
        self.read(value + 260).await
    }

    /// Reads the value at 261.
    async fn jsonrpc_read_261(&self, value: u32) -> This::Reading {
        self.read(value + 261).await
    }

    /// Reads the value at 262.
    async fn jsonrpc_read_262(&self, value: u32) -> This::Reading {
        self.read(value + 262).await
    }

    /// Reads the value at 263.
    async fn jsonrpc_read_263(&self, value: u32) -> This::Reading {
        self.read(value + 263).await
    }

    /// Reads the value at 264.
    async fn jsonrpc_read_264(&self, value: u32) -> This::Reading {
        self.read(value + 264).await
    }

    /// Reads the value at 265.
    async fn jsonrpc_read_265(&self, value: u32) -> This::Reading {
        self.read(value + 265).await
    }

    /// Reads the value at 266.
    async fn jsonrpc_read_266(&self, value: u32) -> This::Reading {
        self.read(value + 266).await
    }

    /// Reads the value at 267.
    async fn jsonrpc_read_267(&self, value: u32) -> This::Reading {
        self.read(value + 267).await
    }

    /// Reads the value at 268.
    async fn jsonrpc_read_268(&self, value: u32) -> This::Reading {
        self.read(value + 268).await
    }

    /// Reads the value at 269.
    async fn jsonrpc_read_269(&self, value: u32) -> This::Reading {
        self.read(value + 269).await
    }

    /// Reads the value at 270.
    async fn jsonrpc_read_270(&self, value: u32) -> This::Reading {
        self.read(value + 270).await
    }

    /// Reads the value at 271.
    async fn jsonrpc_read_271(&self, value: u32) -> This::Reading {
        self.read(value + 271).await
    }

    /// Reads the value at 272.
    async fn jsonrpc_read_272(&self, value: u32) -> This::Reading {
        self.read(value + 272).await
    }

    /// Reads the value at 273.
    async fn jsonrpc_read_273(&self, value: u32) -> This::Reading {
        self.read(value + 273).await
    }

    /// Reads the value at 274.
    async fn jsonrpc_read_274(&self, value: u32) -> This::Reading {
        self.read(value + 274).await
    }

    /// Reads the value at 275.
    async fn jsonrpc_read_275(&self, value: u32) -> This::Reading {
        self.read(value + 275).await
    }

    /// Reads the value at 276.
    async fn jsonrpc_read_276(&self, value: u32) -> This::Reading {
        self.read(value + 276).await
    }

    /// Reads the value at 277.
    async fn jsonrpc_read_277(&self, value: u32) -> This::Reading {
        self.read(value + 277).await
    }

    /// Reads the value at 278.
    async fn jsonrpc_read_278(&self, value: u32) -> This::Reading {
        self.read(value + 278).await
    }

    /// Reads the value at 279.
    async fn jsonrpc_read_279(&self, value: u32) -> This::Reading {
        self.read(value + 279).await
    }

    /// Reads the value at 280.
    async fn jsonrpc_read_280(&self, value: u32) -> This::Reading {
        self.read(value + 280).await
    }

    /// Reads the value at 281.
    async fn jsonrpc_read_281(&self, value: u32) -> This::Reading {
        self.read(value + 281).await
    }

    /// Reads the value at 282.
    async fn jsonrpc_read_282(&self, value: u32) -> This::Reading {
        self.read(value + 282).await
    }

    /// Reads the value at 283.
    async fn jsonrpc_read_283(&self, value: u32) -> This::Reading {
        self.read(value + 283).await
    }

    /// Reads the value at 284.
    async fn jsonrpc_read_284(&self, value: u32) -> This::Reading {
        self.read(value + 284).await
    }

    /// Reads the value at 285.
    async fn jsonrpc_read_285(&self, value: u32) -> This::Reading {
        self.read(value + 285).await
    }

    /// Reads the value at 286.
    async fn jsonrpc_read_286(&self, value: u32) -> This::Reading {
        self.read(value + 286).await
    }

    /// Reads the value at 287.
    async fn jsonrpc_read_287(&self, value: u32) -> This::Reading {
        self.read(value + 287).await
    }

    /// Reads the value at 288.
    async fn jsonrpc_read_288(&self, value: u32) -> This::Reading {
        self.read(value + 288).await
    }

    /// Reads the value at 289.
    async fn jsonrpc_read_289(&self, value: u32) -> This::Reading {
        self.read(value + 289).await
    }

    /// Reads the value at 290.
    async fn jsonrpc_read_290(&self, value: u32) -> This::Reading {
        self.read(value + 290).await
    }

    /// Reads the value at 291.
    async fn jsonrpc_read_291(&self, value: u32) -> This::Reading {
        self.read(value + 291).await
    }

    /// Reads the value at 292.
    async fn jsonrpc_read_292(&self, value: u32) -> This::Reading {
        self.read(value + 292).await
    }

    /// Reads the value at 293.
    async fn jsonrpc_read_293(&self, value: u32) -> This::Reading {
        self.read(value + 293).await
    }

    /// Reads the value at 294.
    async fn jsonrpc_read_294(&self, value: u32) -> This::Reading {
        self.read(value + 294).await
    }

    /// Reads the value at 295.
    async fn jsonrpc_read_295(&self, value: u32) -> This::Reading {
        self.read(value + 295).await
    }

    /// Reads the value at 296.
    async fn jsonrpc_read_296(&self, value: u32) -> This::Reading {
        self.read(value + 296).await
    }

    /// Reads the value at 297.
    async fn jsonrpc_read_297(&self, value: u32) -> This::Reading {
        self.read(value + 297).await
    }

    /// Reads the value at 298.
    async fn jsonrpc_read_298(&self, value: u32) -> This::Reading {
        self.read(value + 298).await
    }

    /// Reads the value at 299.
    async fn jsonrpc_read_299(&self, value: u32) -> This::Reading {
        self.read(value + 299).await
    }

    /// Reads the value at 300.
    async fn jsonrpc_read_300(&self, value: u32) -> This::Reading {
        self.read(value + 300).await
    }

    /// Reads the value at 301.
    async fn jsonrpc_read_301(&self, value: u32) -> This::Reading {
        self.read(value + 301).await
    }

    /// Reads the value at 302.
    async fn jsonrpc_read_302(&self, value: u32) -> This::Reading {
        self.read(value + 302).await
    }

    /// Reads the value at 303.
    async fn jsonrpc_read_303(&self, value: u32) -> This::Reading {
        self.read(value + 303).await
    }

    /// Reads the value at 304.
    async fn jsonrpc_read_304(&self, value: u32) -> This::Reading {
        self.read(value + 304).await
    }

    /// Reads the value at 305.
    async fn jsonrpc_read_305(&self, value: u32) -> This::Reading {
        self.read(value + 305).await
    }

    /// Reads the value at 306.
    async fn jsonrpc_read_306(&self, value: u32) -> This::Reading {
        self.read(value + 306).await
    }

    /// Reads the value at 307.
    async fn jsonrpc_read_307(&self, value: u32) -> This::Reading {
        self.read(value + 307).await
    }

    /// Reads the value at 308.
    async fn jsonrpc_read_308(&self, value: u32) -> This::Reading {
        self.read(value + 308).await
    }

    /// Reads the value at 309.
    async fn jsonrpc_read_309(&self, value: u32) -> This::Reading {
        self.read(value + 309).await
    }

    /// Reads the value at 310.
    async fn jsonrpc_read_310(&self, value: u32) -> This::Reading {
        self.read(value + 310).await
    }

    /// Reads the value at 311.
    async fn jsonrpc_read_311(&self, value: u32) -> This::Reading {
        self.read(value + 311).await
    }

    /// Reads the value at 312.
    async fn jsonrpc_read_312(&self, value: u32) -> This::Reading {
        self.read(value + 312).await
    }

    /// Reads the value at 313.
    async fn jsonrpc_read_313(&self, value: u32) -> This::Reading {
        self.read(value + 313).await
    }

    /// Reads the value at 314.
    async fn jsonrpc_read_314(&self, value: u32) -> This::Reading {
        self.read(value + 314).await
    }

    /// Reads the value at 315.
    async fn jsonrpc_read_315(&self, value: u32) -> This::Reading {
        self.read(value + 315).await
    }

    /// Reads the value at 316.
    async fn jsonrpc_read_316(&self, value: u32) -> This::Reading {
        self.read(value + 316).await
    }

    /// Reads the value at 317.
    async fn jsonrpc_read_317(&self, value: u32) -> This::Reading {
        self.read(value + 317).await
    }

    /// Reads the value at 318.
    async fn jsonrpc_read_318(&self, value: u32) -> This::Reading {
        self.read(value + 318).await
    }

    /// Reads the value at 319.
    async fn jsonrpc_read_319(&self, value: u32) -> This::Reading {
        self.read(value + 319).await
    }

    /// Reads the value at 320.
    async fn jsonrpc_read_320(&self, value: u32) -> This::Reading {
        self.read(value + 320).await
    }

    /// Reads the value at 321.
    async fn jsonrpc_read_321(&self, value: u32) -> This::Reading {
        self.read(value + 321).await
    }

    /// Reads the value at 322.
    async fn jsonrpc_read_322(&self, value: u32) -> This::Reading {
        self.read(value + 322).await
    }

    /// Reads the value at 323.
    async fn jsonrpc_read_323(&self, value: u32) -> This::Reading {
        self.read(value + 323).await
    }

    /// Reads the value at 324.
    async fn jsonrpc_read_324(&self, value: u32) -> This::Reading {
        self.read(value + 324).await
    }

    /// Reads the value at 325.
    async fn jsonrpc_read_325(&self, value: u32) -> This::Reading {
        self.read(value + 325).await
    }

    /// Reads the value at 326.
    async fn jsonrpc_read_326(&self, value: u32) -> This::Reading {
        self.read(value + 326).await
    }

    /// Reads the value at 327.
    async fn jsonrpc_read_327(&self, value: u32) -> This::Reading {
        self.read(value + 327).await
    }

    /// Reads the value at 328.
    async fn jsonrpc_read_328(&self, value: u32) -> This::Reading {
        self.read(value + 328).await
    }

    /// Reads the value at 329.
    async fn jsonrpc_read_329(&self, value: u32) -> This::Reading {
        self.read(value + 329).await
    }

    /// Reads the value at 330.
    async fn jsonrpc_read_330(&self, value: u32) -> This::Reading {
        self.read(value + 330).await
    }

    /// Reads the value at 331.
    async fn jsonrpc_read_331(&self, value: u32) -> This::Reading {
        self.read(value + 331).await
    }

    /// Reads the value at 332.
    async fn jsonrpc_read_332(&self, value: u32) -> This::Reading {
        self.read(value + 332).await
    }

    /// Reads the value at 333.
    async fn jsonrpc_read_333(&self, value: u32) -> This::Reading {
        self.read(value + 333).await
    }

    /// Reads the value at 334.
    async fn jsonrpc_read_334(&self, value: u32) -> This::Reading {
        self.read(value + 334).await
    }

    /// Reads the value at 335.
    async fn jsonrpc_read_335(&self, value: u32) -> This::Reading {
        self.read(value + 335).await
    }

    /// Reads the value at 336.
    async fn jsonrpc_read_336(&self, value: u32) -> This::Reading {
        self.read(value + 336).await
    }

    /// Reads the value at 337.
    async fn jsonrpc_read_337(&self, value: u32) -> This::Reading {
        self.read(value + 337).await
    }

    /// Reads the value at 338.
    async fn jsonrpc_read_338(&self, value: u32) -> This::Reading {
        self.read(value + 338).await
    }

    /// Reads the value at 339.
    async fn jsonrpc_read_339(&self, value: u32) -> This::Reading {
        self.read(value + 339).await
    }

    /// Reads the value at 340.
    async fn jsonrpc_read_340(&self, value: u32) -> This::Reading {
        self.read(value + 340).await
    }

    /// Reads the value at 341.
    async fn jsonrpc_read_341(&self, value: u32) -> This::Reading {
        self.read(value + 341).await
    }

    /// Reads the value at 342.
    async fn jsonrpc_read_342(&self, value: u32) -> This::Reading {
        self.read(value + 342).await
    }

    /// Reads the value at 343.
    async fn jsonrpc_read_343(&self, value: u32) -> This::Reading {
        self.read(value + 343).await
    }

    /// Reads the value at 344.
    async fn jsonrpc_read_344(&self, value: u32) -> This::Reading {
        self.read(value + 344).await
    }

    /// Reads the value at 345.
    async fn jsonrpc_read_345(&self, value: u32) -> This::Reading {
        self.read(value + 345).await
    }

    /// Reads the value at 346.
    async fn jsonrpc_read_346(&self, value: u32) -> This::Reading {
        self.read(value + 346).await
    }

    /// Reads the value at 347.
    async fn jsonrpc_read_347(&self, value: u32) -> This::Reading {
        self.read(value + 347).await
    }

    /// Reads the value at 348.
    async fn jsonrpc_read_348(&self, value: u32) -> This::Reading {
        self.read(value + 348).await
    }

    /// Reads the value at 349.
    async fn jsonrpc_read_349(&self, value: u32) -> This::Reading {
        self.read(value + 349).await
    }

    /// Reads the value at 350.
    async fn jsonrpc_read_350(&self, value: u32) -> This::Reading {
        self.read(value + 350).await
    }

    /// Reads the value at 351.
    async fn jsonrpc_read_351(&self, value: u32) -> This::Reading {
        self.read(value + 351).await
    }

    /// Reads the value at 352.
    async fn jsonrpc_read_352(&self, value: u32) -> This::Reading {
        self.read(value + 352).await
    }

    /// Reads the value at 353.
    async fn jsonrpc_read_353(&self, value: u32) -> This::Reading {
        self.read(value + 353).await
    }

    /// Reads the value at 354.
    async fn jsonrpc_read_354(&self, value: u32) -> This::Reading {
        self.read(value + 354).await
    }

    /// Reads the value at 355.
    async fn jsonrpc_read_355(&self, value: u32) -> This::Reading {
        self.read(value + 355).await
    }

    /// Reads the value at 356.
    async fn jsonrpc_read_356(&self, value: u32) -> This::Reading {
        self.read(value + 356).await
    }

    /// Reads the value at 357.
    async fn jsonrpc_read_357(&self, value: u32) -> This::Reading {
        self.read(value + 357).await
    }

    /// Reads the value at 358.
    async fn jsonrpc_read_358(&self, value: u32) -> This::Reading {
        self.read(value + 358).await
    }

    /// Reads the value at 359.
    async fn jsonrpc_read_359(&self, value: u32) -> This::Reading {
        self.read(value + 359).await
    }

    /// Reads the value at 360.
    async fn jsonrpc_read_360(&self, value: u32) -> This::Reading {
        self.read(value + 360).await
    }

    /// Reads the value at 361.
    async fn jsonrpc_read_361(&self, value: u32) -> This::Reading {
        self.read(value + 361).await
    }

    /// Reads the value at 362.
    async fn jsonrpc_read_362(&self, value: u32) -> This::Reading {
        self.read(value + 362).await
    }

    /// Reads the value at 363.
    async fn jsonrpc_read_363(&self, value: u32) -> This::Reading {
        self.read(value + 363).await
    }

    /// Reads the value at 364.
    async fn jsonrpc_read_364(&self, value: u32) -> This::Reading {
        self.read(value + 364).await
    }

    /// Reads the value at 365.
    async fn jsonrpc_read_365(&self, value: u32) -> This::Reading {
        self.read(value + 365).await
    }

    /// Reads the value at 366.
    async fn jsonrpc_read_366(&self, value: u32) -> This::Reading {
        self.read(value + 366).await
    }

    /// Reads the value at 367.
    async fn jsonrpc_read_367(&self, value: u32) -> This::Reading {
        self.read(value + 367).await
    }

    /// Reads the value at 368.
    async fn jsonrpc_read_368(&self, value: u32) -> This::Reading {
        self.read(value + 368).await
    }

    /// Reads the value at 369.
    async fn jsonrpc_read_369(&self, value: u32) -> This::Reading {
        self.read(value + 369).await
    }

    /// Reads the value at 370.
    async fn jsonrpc_read_370(&self, value: u32) -> This::Reading {
        self.read(value + 370).await
    }

    /// Reads the value at 371.
    async fn jsonrpc_read_371(&self, value: u32) -> This::Reading {
        self.read(value + 371).await
    }

    /// Reads the value at 372.
    async fn jsonrpc_read_372(&self, value: u32) -> This::Reading {
        self.read(value + 372).await
    }

    /// Reads the value at 373.
    async fn jsonrpc_read_373(&self, value: u32) -> This::Reading {
        self.read(value + 373).await
    }

    /// Reads the value at 374.
    async fn jsonrpc_read_374(&self, value: u32) -> This::Reading {
        self.read(value + 374).await
    }

    /// Reads the value at 375.
    async fn jsonrpc_read_375(&self, value: u32) -> This::Reading {
        self.read(value + 375).await
    }

    /// Reads the value at 376.
    async fn jsonrpc_read_376(&self, value: u32) -> This::Reading {
        self.read(value + 376).await
    }

    /// Reads the value at 377.
    async fn jsonrpc_read_377(&self, value: u32) -> This::Reading {
        self.read(value + 377).await
    }

    /// Reads the value at 378.
    async fn jsonrpc_read_378(&self, value: u32) -> This::Reading {
        self.read(value + 378).await
    }

    /// Reads the value at 379.
    async fn jsonrpc_read_379(&self, value: u32) -> This::Reading {
        self.read(value + 379).await
    }

    /// Reads the value at 380.
    async fn jsonrpc_read_380(&self, value: u32) -> This::Reading {
        self.read(value + 380).await
    }

    /// Reads the value at 381.
    async fn jsonrpc_read_381(&self, value: u32) -> This::Reading {
        self.read(value + 381).await
    }

    /// Reads the value at 382.
    async fn jsonrpc_read_382(&self, value: u32) -> This::Reading {
        self.read(value + 382).await
    }

    /// Reads the value at 383.
    async fn jsonrpc_read_383(&self, value: u32) -> This::Reading {
        self.read(value + 383).await
    }

    /// Reads the value at 384.
    async fn jsonrpc_read_384(&self, value: u32) -> This::Reading {
        self.read(value + 384).await
    }

    /// Reads the value at 385.
    async fn jsonrpc_read_385(&self, value: u32) -> This::Reading {
        self.read(value + 385).await
    }

    /// Reads the value at 386.
    async fn jsonrpc_read_386(&self, value: u32) -> This::Reading {
        self.read(value + 386).await
    }

    /// Reads the value at 387.
    async fn jsonrpc_read_387(&self, value: u32) -> This::Reading {
        self.read(value + 387).await
    }

    /// Reads the value at 388.
    async fn jsonrpc_read_388(&self, value: u32) -> This::Reading {
        self.read(value + 388).await
    }

    /// Reads the value at 389.
    async fn jsonrpc_read_389(&self, value: u32) -> This::Reading {
        self.read(value + 389).await
    }

    /// Reads the value at 390.
    async fn jsonrpc_read_390(&self, value: u32) -> This::Reading {
        self.read(value + 390).await
    }

    /// Reads the value at 391.
    async fn jsonrpc_read_391(&self, value: u32) -> This::Reading {
        self.read(value + 391).await
    }

    /// Reads the value at 392.
    async fn jsonrpc_read_392(&self, value: u32) -> This::Reading {
        self.read(value + 392).await
    }

    /// Reads the value at 393.
    async fn jsonrpc_read_393(&self, value: u32) -> This::Reading {
        self.read(value + 393).await
    }

    /// Reads the value at 394.
    async fn jsonrpc_read_394(&self, value: u32) -> This::Reading {
        self.read(value + 394).await
    }

    /// Reads the value at 395.
    async fn jsonrpc_read_395(&self, value: u32) -> This::Reading {
        self.read(value + 395).await
    }

    /// Reads the value at 396.
    async fn jsonrpc_read_396(&self, value: u32) -> This::Reading {
        self.read(value + 396).await
    }

    /// Reads the value at 397.
    async fn jsonrpc_read_397(&self, value: u32) -> This::Reading {
        self.read(value + 397).await
    }

    /// Reads the value at 398.
    async fn jsonrpc_read_398(&self, value: u32) -> This::Reading {
        self.read(value + 398).await
    }

    /// Reads the value at 399.
    async fn jsonrpc_read_399(&self, value: u32) -> This::Reading {
        self.read(value + 399).await
    }

    /// Reads the value at 400.
    async fn jsonrpc_read_400(&self, value: u32) -> This::Reading {
        self.read(value + 400).await
    }

    /// Reads the value at 401.
    async fn jsonrpc_read_401(&self, value: u32) -> This::Reading {
        self.read(value + 401).await
    }

    /// Reads the value at 402.
    async fn jsonrpc_read_402(&self, value: u32) -> This::Reading {
        self.read(value + 402).await
    }

    /// Reads the value at 403.
    async fn jsonrpc_read_403(&self, value: u32) -> This::Reading {
        self.read(value + 403).await
    }

    /// Reads the value at 404.
    async fn jsonrpc_read_404(&self, value: u32) -> This::Reading {
        self.read(value + 404).await
    }

    /// Reads the value at 405.
    async fn jsonrpc_read_405(&self, value: u32) -> This::Reading {
        self.read(value + 405).await
    }

    /// Reads the value at 406.
    async fn jsonrpc_read_406(&self, value: u32) -> This::Reading {
        self.read(value + 406).await
    }

    /// Reads the value at 407.
    async fn jsonrpc_read_407(&self, value: u32) -> This::Reading {
        self.read(value + 407).await
    }

    /// Reads the value at 408.
    async fn jsonrpc_read_408(&self, value: u32) -> This::Reading {
        self.read(value + 408).await
    }

    /// Reads the value at 409.
    async fn jsonrpc_read_409(&self, value: u32) -> This::Reading {
        self.read(value + 409).await
    }

    /// Reads the value at 410.
    async fn jsonrpc_read_410(&self, value: u32) -> This::Reading {
        self.read(value + 410).await
    }

    /// Reads the value at 411.
    async fn jsonrpc_read_411(&self, value: u32) -> This::Reading {
        self.read(value + 411).await
    }

    /// Reads the value at 412.
    async fn jsonrpc_read_412(&self, value: u32) -> This::Reading {
        self.read(value + 412).await
    }

    /// Reads the value at 413.
    async fn jsonrpc_read_413(&self, value: u32) -> This::Reading {
        self.read(value + 413).await
    }

    /// Reads the value at 414.
    async fn jsonrpc_read_414(&self, value: u32) -> This::Reading {
        self.read(value + 414).await
    }

    /// Reads the value at 415.
    async fn jsonrpc_read_415(&self, value: u32) -> This::Reading {
        self.read(value + 415).await
    }

    /// Reads the value at 416.
    async fn jsonrpc_read_416(&self, value: u32) -> This::Reading {
        self.read(value + 416).await
    }

    /// Reads the value at 417.
    async fn jsonrpc_read_417(&self, value: u32) -> This::Reading {
        self.read(value + 417).await
    }

    /// Reads the value at 418.
    async fn jsonrpc_read_418(&self, value: u32) -> This::Reading {
        self.read(value + 418).await
    }

    /// Reads the value at 419.
    async fn jsonrpc_read_419(&self, value: u32) -> This::Reading {
        self.read(value + 419).await
    }

    /// Reads the value at 420.
    async fn jsonrpc_read_420(&self, value: u32) -> This::Reading {
        self.read(value + 420).await
    }

    /// Reads the value at 421.
    async fn jsonrpc_read_421(&self, value: u32) -> This::Reading {
        self.read(value + 421).await
    }

    /// Reads the value at 422.
    async fn jsonrpc_read_422(&self, value: u32) -> This::Reading {
        self.read(value + 422).await
    }

    /// Reads the value at 423.
    async fn jsonrpc_read_423(&self, value: u32) -> This::Reading {
        self.read(value + 423).await
    }

    /// Reads the value at 424.
    async fn jsonrpc_read_424(&self, value: u32) -> This::Reading {
        self.read(value + 424).await
    }

    /// Reads the value at 425.
    async fn jsonrpc_read_425(&self, value: u32) -> This::Reading {
        self.read(value + 425).await
    }

    /// Reads the value at 426.
    async fn jsonrpc_read_426(&self, value: u32) -> This::Reading {
        self.read(value + 426).await
    }

    /// Reads the value at 427.
    async fn jsonrpc_read_427(&self, value: u32) -> This::Reading {
        self.read(value + 427).await
    }

    /// Reads the value at 428.
    async fn jsonrpc_read_428(&self, value: u32) -> This::Reading {
        self.read(value + 428).await
    }

    /// Reads the value at 429.
    async fn jsonrpc_read_429(&self, value: u32) -> This::Reading {
        self.read(value + 429).await
    }

    /// Reads the value at 430.
    async fn jsonrpc_read_430(&self, value: u32) -> This::Reading {
        self.read(value + 430).await
    }

    /// Reads the value at 431.
    async fn jsonrpc_read_431(&self, value: u32) -> This::Reading {
        self.read(value + 431).await
    }

    /// Reads the value at 432.
    async fn jsonrpc_read_432(&self, value: u32) -> This::Reading {
        self.read(value + 432).await
    }

    /// Reads the value at 433.
    async fn jsonrpc_read_433(&self, value: u32) -> This::Reading {
        self.read(value + 433).await
    }

    /// Reads the value at 434.
    async fn jsonrpc_read_434(&self, value: u32) -> This::Reading {
        self.read(value + 434).await
    }

    /// Reads the value at 435.
    async fn jsonrpc_read_435(&self, value: u32) -> This::Reading {
        self.read(value + 435).await
    }

    /// Reads the value at 436.
    async fn jsonrpc_read_436(&self, value: u32) -> This::Reading {
        self.read(value + 436).await
    }

    /// Reads the value at 437.
    async fn jsonrpc_read_437(&self, value: u32) -> This::Reading {
        self.read(value + 437).await
    }

    /// Reads the value at 438.
    async fn jsonrpc_read_438(&self, value: u32) -> This::Reading {
        self.read(value + 438).await
    }

    /// Reads the value at 439.
    async fn jsonrpc_read_439(&self, value: u32) -> This::Reading {
        self.read(value + 439).await
    }

    /// Reads the value at 440.
    async fn jsonrpc_read_440(&self, value: u32) -> This::Reading {
        self.read(value + 440).await
    }

    /// Reads the value at 441.
    async fn jsonrpc_read_441(&self, value: u32) -> This::Reading {
        self.read(value + 441).await
    }

    /// Reads the value at 442.
    async fn jsonrpc_read_442(&self, value: u32) -> This::Reading {
        self.read(value + 442).await
    }

    /// Reads the value at 443.
    async fn jsonrpc_read_443(&self, value: u32) -> This::Reading {
        self.read(value + 443).await
    }

    /// Reads the value at 444.
    async fn jsonrpc_read_444(&self, value: u32) -> This::Reading {
        self.read(value + 444).await
    }

    /// Reads the value at 445.
    async fn jsonrpc_read_445(&self, value: u32) -> This::Reading {
        self.read(value + 445).await
    }

    /// Reads the value at 446.
    async fn jsonrpc_read_446(&self, value: u32) -> This::Reading {
        self.read(value + 446).await
    }

    /// Reads the value at 447.
    async fn jsonrpc_read_447(&self, value: u32) -> This::Reading {
        self.read(value + 447).await
    }

    /// Reads the value at 448.
    async fn jsonrpc_read_448(&self, value: u32) -> This::Reading {
        self.read(value + 448).await
    }

    /// Reads the value at 449.
    async fn jsonrpc_read_449(&self, value: u32) -> This::Reading {
        self.read(value + 449).await
    }

    /// Reads the value at 450.
    async fn jsonrpc_read_450(&self, value: u32) -> This::Reading {
        self.read(value + 450).await
    }

    /// Reads the value at 451.
    async fn jsonrpc_read_451(&self, value: u32) -> This::Reading {
        self.read(value + 451).await
    }

    /// Reads the value at 452.
    async fn jsonrpc_read_452(&self, value: u32) -> This::Reading {
        self.read(value + 452).await
    }

    /// Reads the value at 453.
    async fn jsonrpc_read_453(&self, value: u32) -> This::Reading {
        self.read(value + 453).await
    }

    /// Reads the value at 454.
    async fn jsonrpc_read_454(&self, value: u32) -> This::Reading {
        self.read(value + 454).await
    }

    /// Reads the value at 455.
    async fn jsonrpc_read_455(&self, value: u32) -> This::Reading {
        self.read(value + 455).await
    }

    /// Reads the value at 456.
    async fn jsonrpc_read_456(&self, value: u32) -> This::Reading {
        self.read(value + 456).await
    }

    /// Reads the value at 457.
    async fn jsonrpc_read_457(&self, value: u32) -> This::Reading {
        self.read(value + 457).await
    }

    /// Reads the value at 458.
    async fn jsonrpc_read_458(&self, value: u32) -> This::Reading {
        self.read(value + 458).await
    }

    /// Reads the value at 459.
    async fn jsonrpc_read_459(&self, value: u32) -> This::Reading {
        self.read(value + 459).await
    }

    /// Reads the value at 460.
    async fn jsonrpc_read_460(&self, value: u32) -> This::Reading {
        self.read(value + 460).await
    }

    /// Reads the value at 461.
    async fn jsonrpc_read_461(&self, value: u32) -> This::Reading {
        self.read(value + 461).await
    }

    /// Reads the value at 462.
    async fn jsonrpc_read_462(&self, value: u32) -> This::Reading {
        self.read(value + 462).await
    }

    /// Reads the value at 463.
    async fn jsonrpc_read_463(&self, value: u32) -> This::Reading {
        self.read(value + 463).await
    }

    /// Reads the value at 464.
    async fn jsonrpc_read_464(&self, value: u32) -> This::Reading {
        self.read(value + 464).await
    }

    /// Reads the value at 465.
    async fn jsonrpc_read_465(&self, value: u32) -> This::Reading {
        self.read(value + 465).await
    }

    /// Reads the value at 466.
    async fn jsonrpc_read_466(&self, value: u32) -> This::Reading {
        self.read(value + 466).await
    }

    /// Reads the value at 467.
    async fn jsonrpc_read_467(&self, value: u32) -> This::Reading {
        self.read(value + 467).await
    }

    /// Reads the value at 468.
    async fn jsonrpc_read_468(&self, value: u32) -> This::Reading {
        self.read(value + 468).await
    }

    /// Reads the value at 469.
    async fn jsonrpc_read_469(&self, value: u32) -> This::Reading {
        self.read(value + 469).await
    }

    /// Reads the value at 470.
    async fn jsonrpc_read_470(&self, value: u32) -> This::Reading {
        self.read(value + 470).await
    }

    /// Reads the value at 471.
    async fn jsonrpc_read_471(&self, value: u32) -> This::Reading {
        self.read(value + 471).await
    }

    /// Reads the value at 472.
    async fn jsonrpc_read_472(&self, value: u32) -> This::Reading {
        self.read(value + 472).await
    }

    /// Reads the value at 473.
    async fn jsonrpc_read_473(&self, value: u32) -> This::Reading {
        self.read(value + 473).await
    }

    /// Reads the value at 474.
    async fn jsonrpc_read_474(&self, value: u32) -> This::Reading {
        self.read(value + 474).await
    }

    /// Reads the value at 475.
    async fn jsonrpc_read_475(&self, value: u32) -> This::Reading {
        self.read(value + 475).await
    }

    /// Reads the value at 476.
    async fn jsonrpc_read_476(&self, value: u32) -> This::Reading {
        self.read(value + 476).await
    }

    /// Reads the value at 477.
    async fn jsonrpc_read_477(&self, value: u32) -> This::Reading {
        self.read(value + 477).await
    }

    /// Reads the value at 478.
    async fn jsonrpc_read_478(&self, value: u32) -> This::Reading {
        self.read(value + 478).await
    }

    /// Reads the value at 479.
    async fn jsonrpc_read_479(&self, value: u32) -> This::Reading {
        self.read(value + 479).await
    }

    /// Reads the value at 480.
    async fn jsonrpc_read_480(&self, value: u32) -> This::Reading {
        self.read(value + 480).await
    }

    /// Reads the value at 481.
    async fn jsonrpc_read_481(&self, value: u32) -> This::Reading {
        self.read(value + 481).await
    }

    /// Reads the value at 482.
    async fn jsonrpc_read_482(&self, value: u32) -> This::Reading {
        self.read(value + 482).await
    }

    /// Reads the value at 483.
    async fn jsonrpc_read_483(&self, value: u32) -> This::Reading {
        self.read(value + 483).await
    }

    /// Reads the value at 484.
    async fn jsonrpc_read_484(&self, value: u32) -> This::Reading {
        self.read(value + 484).await
    }

    /// Reads the value at 485.
    async fn jsonrpc_read_485(&self, value: u32) -> This::Reading {
        self.read(value + 485).await
    }

    /// Reads the value at 486.
    async fn jsonrpc_read_486(&self, value: u32) -> This::Reading {
        self.read(value + 486).await
    }

    /// Reads the value at 487.
    async fn jsonrpc_read_487(&self, value: u32) -> This::Reading {
        self.read(value + 487).await
    }

    /// Reads the value at 488.
    async fn jsonrpc_read_488(&self, value: u32) -> This::Reading {
        self.read(value + 488).await
    }

    /// Reads the value at 489.
    async fn jsonrpc_read_489(&self, value: u32) -> This::Reading {
        self.read(value + 489).await
    }

    /// Reads the value at 490.
    async fn jsonrpc_read_490(&self, value: u32) -> This::Reading {
        self.read(value + 490).await
    }

    /// Reads the value at 491.
    async fn jsonrpc_read_491(&self, value: u32) -> This::Reading {
        self.read(value + 491).await
    }

    /// Reads the value at 492.
    async fn jsonrpc_read_492(&self, value: u32) -> This::Reading {
        self.read(value + 492).await
    }

    /// Reads the value at 493.
    async fn jsonrpc_read_493(&self, value: u32) -> This::Reading {
        self.read(value + 493).await
    }

    /// Reads the value at 494.
    async fn jsonrpc_read_494(&self, value: u32) -> This::Reading {
        self.read(value + 494).await
    }

    /// Reads the value at 495.
    async fn jsonrpc_read_495(&self, value: u32) -> This::Reading {
        self.read(value + 495).await
    }

    /// Reads the value at 496.
    async fn jsonrpc_read_496(&self, value: u32) -> This::Reading {
        self.read(value + 496).await
    }

    /// Reads the value at 497.
    async fn jsonrpc_read_497(&self, value: u32) -> This::Reading {
        self.read(value + 497).await
    }

    /// Reads the value at 498.
    async fn jsonrpc_read_498(&self, value: u32) -> This::Reading {
        self.read(value + 498).await
    }

    /// Reads the value at 499.
    async fn jsonrpc_read_499(&self, value: u32) -> This::Reading {
        self.read(value + 499).await
    }
}

/// Declares every reading in one chain, which is the shape being guarded.
#[ext(name = ManyRpcExt, defunc(via = jsonrpc))]
pub impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares all 500 methods.
    fn many_rpc<Alg>(&self)
    where
        Alg: ReadingAlg,
    {
        self.methods()
            .method("read_0", self.op(Alg::jsonrpc_read_0))
            .method("read_1", self.op(Alg::jsonrpc_read_1))
            .method("read_2", self.op(Alg::jsonrpc_read_2))
            .method("read_3", self.op(Alg::jsonrpc_read_3))
            .method("read_4", self.op(Alg::jsonrpc_read_4))
            .method("read_5", self.op(Alg::jsonrpc_read_5))
            .method("read_6", self.op(Alg::jsonrpc_read_6))
            .method("read_7", self.op(Alg::jsonrpc_read_7))
            .method("read_8", self.op(Alg::jsonrpc_read_8))
            .method("read_9", self.op(Alg::jsonrpc_read_9))
            .method("read_10", self.op(Alg::jsonrpc_read_10))
            .method("read_11", self.op(Alg::jsonrpc_read_11))
            .method("read_12", self.op(Alg::jsonrpc_read_12))
            .method("read_13", self.op(Alg::jsonrpc_read_13))
            .method("read_14", self.op(Alg::jsonrpc_read_14))
            .method("read_15", self.op(Alg::jsonrpc_read_15))
            .method("read_16", self.op(Alg::jsonrpc_read_16))
            .method("read_17", self.op(Alg::jsonrpc_read_17))
            .method("read_18", self.op(Alg::jsonrpc_read_18))
            .method("read_19", self.op(Alg::jsonrpc_read_19))
            .method("read_20", self.op(Alg::jsonrpc_read_20))
            .method("read_21", self.op(Alg::jsonrpc_read_21))
            .method("read_22", self.op(Alg::jsonrpc_read_22))
            .method("read_23", self.op(Alg::jsonrpc_read_23))
            .method("read_24", self.op(Alg::jsonrpc_read_24))
            .method("read_25", self.op(Alg::jsonrpc_read_25))
            .method("read_26", self.op(Alg::jsonrpc_read_26))
            .method("read_27", self.op(Alg::jsonrpc_read_27))
            .method("read_28", self.op(Alg::jsonrpc_read_28))
            .method("read_29", self.op(Alg::jsonrpc_read_29))
            .method("read_30", self.op(Alg::jsonrpc_read_30))
            .method("read_31", self.op(Alg::jsonrpc_read_31))
            .method("read_32", self.op(Alg::jsonrpc_read_32))
            .method("read_33", self.op(Alg::jsonrpc_read_33))
            .method("read_34", self.op(Alg::jsonrpc_read_34))
            .method("read_35", self.op(Alg::jsonrpc_read_35))
            .method("read_36", self.op(Alg::jsonrpc_read_36))
            .method("read_37", self.op(Alg::jsonrpc_read_37))
            .method("read_38", self.op(Alg::jsonrpc_read_38))
            .method("read_39", self.op(Alg::jsonrpc_read_39))
            .method("read_40", self.op(Alg::jsonrpc_read_40))
            .method("read_41", self.op(Alg::jsonrpc_read_41))
            .method("read_42", self.op(Alg::jsonrpc_read_42))
            .method("read_43", self.op(Alg::jsonrpc_read_43))
            .method("read_44", self.op(Alg::jsonrpc_read_44))
            .method("read_45", self.op(Alg::jsonrpc_read_45))
            .method("read_46", self.op(Alg::jsonrpc_read_46))
            .method("read_47", self.op(Alg::jsonrpc_read_47))
            .method("read_48", self.op(Alg::jsonrpc_read_48))
            .method("read_49", self.op(Alg::jsonrpc_read_49))
            .method("read_50", self.op(Alg::jsonrpc_read_50))
            .method("read_51", self.op(Alg::jsonrpc_read_51))
            .method("read_52", self.op(Alg::jsonrpc_read_52))
            .method("read_53", self.op(Alg::jsonrpc_read_53))
            .method("read_54", self.op(Alg::jsonrpc_read_54))
            .method("read_55", self.op(Alg::jsonrpc_read_55))
            .method("read_56", self.op(Alg::jsonrpc_read_56))
            .method("read_57", self.op(Alg::jsonrpc_read_57))
            .method("read_58", self.op(Alg::jsonrpc_read_58))
            .method("read_59", self.op(Alg::jsonrpc_read_59))
            .method("read_60", self.op(Alg::jsonrpc_read_60))
            .method("read_61", self.op(Alg::jsonrpc_read_61))
            .method("read_62", self.op(Alg::jsonrpc_read_62))
            .method("read_63", self.op(Alg::jsonrpc_read_63))
            .method("read_64", self.op(Alg::jsonrpc_read_64))
            .method("read_65", self.op(Alg::jsonrpc_read_65))
            .method("read_66", self.op(Alg::jsonrpc_read_66))
            .method("read_67", self.op(Alg::jsonrpc_read_67))
            .method("read_68", self.op(Alg::jsonrpc_read_68))
            .method("read_69", self.op(Alg::jsonrpc_read_69))
            .method("read_70", self.op(Alg::jsonrpc_read_70))
            .method("read_71", self.op(Alg::jsonrpc_read_71))
            .method("read_72", self.op(Alg::jsonrpc_read_72))
            .method("read_73", self.op(Alg::jsonrpc_read_73))
            .method("read_74", self.op(Alg::jsonrpc_read_74))
            .method("read_75", self.op(Alg::jsonrpc_read_75))
            .method("read_76", self.op(Alg::jsonrpc_read_76))
            .method("read_77", self.op(Alg::jsonrpc_read_77))
            .method("read_78", self.op(Alg::jsonrpc_read_78))
            .method("read_79", self.op(Alg::jsonrpc_read_79))
            .method("read_80", self.op(Alg::jsonrpc_read_80))
            .method("read_81", self.op(Alg::jsonrpc_read_81))
            .method("read_82", self.op(Alg::jsonrpc_read_82))
            .method("read_83", self.op(Alg::jsonrpc_read_83))
            .method("read_84", self.op(Alg::jsonrpc_read_84))
            .method("read_85", self.op(Alg::jsonrpc_read_85))
            .method("read_86", self.op(Alg::jsonrpc_read_86))
            .method("read_87", self.op(Alg::jsonrpc_read_87))
            .method("read_88", self.op(Alg::jsonrpc_read_88))
            .method("read_89", self.op(Alg::jsonrpc_read_89))
            .method("read_90", self.op(Alg::jsonrpc_read_90))
            .method("read_91", self.op(Alg::jsonrpc_read_91))
            .method("read_92", self.op(Alg::jsonrpc_read_92))
            .method("read_93", self.op(Alg::jsonrpc_read_93))
            .method("read_94", self.op(Alg::jsonrpc_read_94))
            .method("read_95", self.op(Alg::jsonrpc_read_95))
            .method("read_96", self.op(Alg::jsonrpc_read_96))
            .method("read_97", self.op(Alg::jsonrpc_read_97))
            .method("read_98", self.op(Alg::jsonrpc_read_98))
            .method("read_99", self.op(Alg::jsonrpc_read_99))
            .method("read_100", self.op(Alg::jsonrpc_read_100))
            .method("read_101", self.op(Alg::jsonrpc_read_101))
            .method("read_102", self.op(Alg::jsonrpc_read_102))
            .method("read_103", self.op(Alg::jsonrpc_read_103))
            .method("read_104", self.op(Alg::jsonrpc_read_104))
            .method("read_105", self.op(Alg::jsonrpc_read_105))
            .method("read_106", self.op(Alg::jsonrpc_read_106))
            .method("read_107", self.op(Alg::jsonrpc_read_107))
            .method("read_108", self.op(Alg::jsonrpc_read_108))
            .method("read_109", self.op(Alg::jsonrpc_read_109))
            .method("read_110", self.op(Alg::jsonrpc_read_110))
            .method("read_111", self.op(Alg::jsonrpc_read_111))
            .method("read_112", self.op(Alg::jsonrpc_read_112))
            .method("read_113", self.op(Alg::jsonrpc_read_113))
            .method("read_114", self.op(Alg::jsonrpc_read_114))
            .method("read_115", self.op(Alg::jsonrpc_read_115))
            .method("read_116", self.op(Alg::jsonrpc_read_116))
            .method("read_117", self.op(Alg::jsonrpc_read_117))
            .method("read_118", self.op(Alg::jsonrpc_read_118))
            .method("read_119", self.op(Alg::jsonrpc_read_119))
            .method("read_120", self.op(Alg::jsonrpc_read_120))
            .method("read_121", self.op(Alg::jsonrpc_read_121))
            .method("read_122", self.op(Alg::jsonrpc_read_122))
            .method("read_123", self.op(Alg::jsonrpc_read_123))
            .method("read_124", self.op(Alg::jsonrpc_read_124))
            .method("read_125", self.op(Alg::jsonrpc_read_125))
            .method("read_126", self.op(Alg::jsonrpc_read_126))
            .method("read_127", self.op(Alg::jsonrpc_read_127))
            .method("read_128", self.op(Alg::jsonrpc_read_128))
            .method("read_129", self.op(Alg::jsonrpc_read_129))
            .method("read_130", self.op(Alg::jsonrpc_read_130))
            .method("read_131", self.op(Alg::jsonrpc_read_131))
            .method("read_132", self.op(Alg::jsonrpc_read_132))
            .method("read_133", self.op(Alg::jsonrpc_read_133))
            .method("read_134", self.op(Alg::jsonrpc_read_134))
            .method("read_135", self.op(Alg::jsonrpc_read_135))
            .method("read_136", self.op(Alg::jsonrpc_read_136))
            .method("read_137", self.op(Alg::jsonrpc_read_137))
            .method("read_138", self.op(Alg::jsonrpc_read_138))
            .method("read_139", self.op(Alg::jsonrpc_read_139))
            .method("read_140", self.op(Alg::jsonrpc_read_140))
            .method("read_141", self.op(Alg::jsonrpc_read_141))
            .method("read_142", self.op(Alg::jsonrpc_read_142))
            .method("read_143", self.op(Alg::jsonrpc_read_143))
            .method("read_144", self.op(Alg::jsonrpc_read_144))
            .method("read_145", self.op(Alg::jsonrpc_read_145))
            .method("read_146", self.op(Alg::jsonrpc_read_146))
            .method("read_147", self.op(Alg::jsonrpc_read_147))
            .method("read_148", self.op(Alg::jsonrpc_read_148))
            .method("read_149", self.op(Alg::jsonrpc_read_149))
            .method("read_150", self.op(Alg::jsonrpc_read_150))
            .method("read_151", self.op(Alg::jsonrpc_read_151))
            .method("read_152", self.op(Alg::jsonrpc_read_152))
            .method("read_153", self.op(Alg::jsonrpc_read_153))
            .method("read_154", self.op(Alg::jsonrpc_read_154))
            .method("read_155", self.op(Alg::jsonrpc_read_155))
            .method("read_156", self.op(Alg::jsonrpc_read_156))
            .method("read_157", self.op(Alg::jsonrpc_read_157))
            .method("read_158", self.op(Alg::jsonrpc_read_158))
            .method("read_159", self.op(Alg::jsonrpc_read_159))
            .method("read_160", self.op(Alg::jsonrpc_read_160))
            .method("read_161", self.op(Alg::jsonrpc_read_161))
            .method("read_162", self.op(Alg::jsonrpc_read_162))
            .method("read_163", self.op(Alg::jsonrpc_read_163))
            .method("read_164", self.op(Alg::jsonrpc_read_164))
            .method("read_165", self.op(Alg::jsonrpc_read_165))
            .method("read_166", self.op(Alg::jsonrpc_read_166))
            .method("read_167", self.op(Alg::jsonrpc_read_167))
            .method("read_168", self.op(Alg::jsonrpc_read_168))
            .method("read_169", self.op(Alg::jsonrpc_read_169))
            .method("read_170", self.op(Alg::jsonrpc_read_170))
            .method("read_171", self.op(Alg::jsonrpc_read_171))
            .method("read_172", self.op(Alg::jsonrpc_read_172))
            .method("read_173", self.op(Alg::jsonrpc_read_173))
            .method("read_174", self.op(Alg::jsonrpc_read_174))
            .method("read_175", self.op(Alg::jsonrpc_read_175))
            .method("read_176", self.op(Alg::jsonrpc_read_176))
            .method("read_177", self.op(Alg::jsonrpc_read_177))
            .method("read_178", self.op(Alg::jsonrpc_read_178))
            .method("read_179", self.op(Alg::jsonrpc_read_179))
            .method("read_180", self.op(Alg::jsonrpc_read_180))
            .method("read_181", self.op(Alg::jsonrpc_read_181))
            .method("read_182", self.op(Alg::jsonrpc_read_182))
            .method("read_183", self.op(Alg::jsonrpc_read_183))
            .method("read_184", self.op(Alg::jsonrpc_read_184))
            .method("read_185", self.op(Alg::jsonrpc_read_185))
            .method("read_186", self.op(Alg::jsonrpc_read_186))
            .method("read_187", self.op(Alg::jsonrpc_read_187))
            .method("read_188", self.op(Alg::jsonrpc_read_188))
            .method("read_189", self.op(Alg::jsonrpc_read_189))
            .method("read_190", self.op(Alg::jsonrpc_read_190))
            .method("read_191", self.op(Alg::jsonrpc_read_191))
            .method("read_192", self.op(Alg::jsonrpc_read_192))
            .method("read_193", self.op(Alg::jsonrpc_read_193))
            .method("read_194", self.op(Alg::jsonrpc_read_194))
            .method("read_195", self.op(Alg::jsonrpc_read_195))
            .method("read_196", self.op(Alg::jsonrpc_read_196))
            .method("read_197", self.op(Alg::jsonrpc_read_197))
            .method("read_198", self.op(Alg::jsonrpc_read_198))
            .method("read_199", self.op(Alg::jsonrpc_read_199))
            .method("read_200", self.op(Alg::jsonrpc_read_200))
            .method("read_201", self.op(Alg::jsonrpc_read_201))
            .method("read_202", self.op(Alg::jsonrpc_read_202))
            .method("read_203", self.op(Alg::jsonrpc_read_203))
            .method("read_204", self.op(Alg::jsonrpc_read_204))
            .method("read_205", self.op(Alg::jsonrpc_read_205))
            .method("read_206", self.op(Alg::jsonrpc_read_206))
            .method("read_207", self.op(Alg::jsonrpc_read_207))
            .method("read_208", self.op(Alg::jsonrpc_read_208))
            .method("read_209", self.op(Alg::jsonrpc_read_209))
            .method("read_210", self.op(Alg::jsonrpc_read_210))
            .method("read_211", self.op(Alg::jsonrpc_read_211))
            .method("read_212", self.op(Alg::jsonrpc_read_212))
            .method("read_213", self.op(Alg::jsonrpc_read_213))
            .method("read_214", self.op(Alg::jsonrpc_read_214))
            .method("read_215", self.op(Alg::jsonrpc_read_215))
            .method("read_216", self.op(Alg::jsonrpc_read_216))
            .method("read_217", self.op(Alg::jsonrpc_read_217))
            .method("read_218", self.op(Alg::jsonrpc_read_218))
            .method("read_219", self.op(Alg::jsonrpc_read_219))
            .method("read_220", self.op(Alg::jsonrpc_read_220))
            .method("read_221", self.op(Alg::jsonrpc_read_221))
            .method("read_222", self.op(Alg::jsonrpc_read_222))
            .method("read_223", self.op(Alg::jsonrpc_read_223))
            .method("read_224", self.op(Alg::jsonrpc_read_224))
            .method("read_225", self.op(Alg::jsonrpc_read_225))
            .method("read_226", self.op(Alg::jsonrpc_read_226))
            .method("read_227", self.op(Alg::jsonrpc_read_227))
            .method("read_228", self.op(Alg::jsonrpc_read_228))
            .method("read_229", self.op(Alg::jsonrpc_read_229))
            .method("read_230", self.op(Alg::jsonrpc_read_230))
            .method("read_231", self.op(Alg::jsonrpc_read_231))
            .method("read_232", self.op(Alg::jsonrpc_read_232))
            .method("read_233", self.op(Alg::jsonrpc_read_233))
            .method("read_234", self.op(Alg::jsonrpc_read_234))
            .method("read_235", self.op(Alg::jsonrpc_read_235))
            .method("read_236", self.op(Alg::jsonrpc_read_236))
            .method("read_237", self.op(Alg::jsonrpc_read_237))
            .method("read_238", self.op(Alg::jsonrpc_read_238))
            .method("read_239", self.op(Alg::jsonrpc_read_239))
            .method("read_240", self.op(Alg::jsonrpc_read_240))
            .method("read_241", self.op(Alg::jsonrpc_read_241))
            .method("read_242", self.op(Alg::jsonrpc_read_242))
            .method("read_243", self.op(Alg::jsonrpc_read_243))
            .method("read_244", self.op(Alg::jsonrpc_read_244))
            .method("read_245", self.op(Alg::jsonrpc_read_245))
            .method("read_246", self.op(Alg::jsonrpc_read_246))
            .method("read_247", self.op(Alg::jsonrpc_read_247))
            .method("read_248", self.op(Alg::jsonrpc_read_248))
            .method("read_249", self.op(Alg::jsonrpc_read_249))
            .method("read_250", self.op(Alg::jsonrpc_read_250))
            .method("read_251", self.op(Alg::jsonrpc_read_251))
            .method("read_252", self.op(Alg::jsonrpc_read_252))
            .method("read_253", self.op(Alg::jsonrpc_read_253))
            .method("read_254", self.op(Alg::jsonrpc_read_254))
            .method("read_255", self.op(Alg::jsonrpc_read_255))
            .method("read_256", self.op(Alg::jsonrpc_read_256))
            .method("read_257", self.op(Alg::jsonrpc_read_257))
            .method("read_258", self.op(Alg::jsonrpc_read_258))
            .method("read_259", self.op(Alg::jsonrpc_read_259))
            .method("read_260", self.op(Alg::jsonrpc_read_260))
            .method("read_261", self.op(Alg::jsonrpc_read_261))
            .method("read_262", self.op(Alg::jsonrpc_read_262))
            .method("read_263", self.op(Alg::jsonrpc_read_263))
            .method("read_264", self.op(Alg::jsonrpc_read_264))
            .method("read_265", self.op(Alg::jsonrpc_read_265))
            .method("read_266", self.op(Alg::jsonrpc_read_266))
            .method("read_267", self.op(Alg::jsonrpc_read_267))
            .method("read_268", self.op(Alg::jsonrpc_read_268))
            .method("read_269", self.op(Alg::jsonrpc_read_269))
            .method("read_270", self.op(Alg::jsonrpc_read_270))
            .method("read_271", self.op(Alg::jsonrpc_read_271))
            .method("read_272", self.op(Alg::jsonrpc_read_272))
            .method("read_273", self.op(Alg::jsonrpc_read_273))
            .method("read_274", self.op(Alg::jsonrpc_read_274))
            .method("read_275", self.op(Alg::jsonrpc_read_275))
            .method("read_276", self.op(Alg::jsonrpc_read_276))
            .method("read_277", self.op(Alg::jsonrpc_read_277))
            .method("read_278", self.op(Alg::jsonrpc_read_278))
            .method("read_279", self.op(Alg::jsonrpc_read_279))
            .method("read_280", self.op(Alg::jsonrpc_read_280))
            .method("read_281", self.op(Alg::jsonrpc_read_281))
            .method("read_282", self.op(Alg::jsonrpc_read_282))
            .method("read_283", self.op(Alg::jsonrpc_read_283))
            .method("read_284", self.op(Alg::jsonrpc_read_284))
            .method("read_285", self.op(Alg::jsonrpc_read_285))
            .method("read_286", self.op(Alg::jsonrpc_read_286))
            .method("read_287", self.op(Alg::jsonrpc_read_287))
            .method("read_288", self.op(Alg::jsonrpc_read_288))
            .method("read_289", self.op(Alg::jsonrpc_read_289))
            .method("read_290", self.op(Alg::jsonrpc_read_290))
            .method("read_291", self.op(Alg::jsonrpc_read_291))
            .method("read_292", self.op(Alg::jsonrpc_read_292))
            .method("read_293", self.op(Alg::jsonrpc_read_293))
            .method("read_294", self.op(Alg::jsonrpc_read_294))
            .method("read_295", self.op(Alg::jsonrpc_read_295))
            .method("read_296", self.op(Alg::jsonrpc_read_296))
            .method("read_297", self.op(Alg::jsonrpc_read_297))
            .method("read_298", self.op(Alg::jsonrpc_read_298))
            .method("read_299", self.op(Alg::jsonrpc_read_299))
            .method("read_300", self.op(Alg::jsonrpc_read_300))
            .method("read_301", self.op(Alg::jsonrpc_read_301))
            .method("read_302", self.op(Alg::jsonrpc_read_302))
            .method("read_303", self.op(Alg::jsonrpc_read_303))
            .method("read_304", self.op(Alg::jsonrpc_read_304))
            .method("read_305", self.op(Alg::jsonrpc_read_305))
            .method("read_306", self.op(Alg::jsonrpc_read_306))
            .method("read_307", self.op(Alg::jsonrpc_read_307))
            .method("read_308", self.op(Alg::jsonrpc_read_308))
            .method("read_309", self.op(Alg::jsonrpc_read_309))
            .method("read_310", self.op(Alg::jsonrpc_read_310))
            .method("read_311", self.op(Alg::jsonrpc_read_311))
            .method("read_312", self.op(Alg::jsonrpc_read_312))
            .method("read_313", self.op(Alg::jsonrpc_read_313))
            .method("read_314", self.op(Alg::jsonrpc_read_314))
            .method("read_315", self.op(Alg::jsonrpc_read_315))
            .method("read_316", self.op(Alg::jsonrpc_read_316))
            .method("read_317", self.op(Alg::jsonrpc_read_317))
            .method("read_318", self.op(Alg::jsonrpc_read_318))
            .method("read_319", self.op(Alg::jsonrpc_read_319))
            .method("read_320", self.op(Alg::jsonrpc_read_320))
            .method("read_321", self.op(Alg::jsonrpc_read_321))
            .method("read_322", self.op(Alg::jsonrpc_read_322))
            .method("read_323", self.op(Alg::jsonrpc_read_323))
            .method("read_324", self.op(Alg::jsonrpc_read_324))
            .method("read_325", self.op(Alg::jsonrpc_read_325))
            .method("read_326", self.op(Alg::jsonrpc_read_326))
            .method("read_327", self.op(Alg::jsonrpc_read_327))
            .method("read_328", self.op(Alg::jsonrpc_read_328))
            .method("read_329", self.op(Alg::jsonrpc_read_329))
            .method("read_330", self.op(Alg::jsonrpc_read_330))
            .method("read_331", self.op(Alg::jsonrpc_read_331))
            .method("read_332", self.op(Alg::jsonrpc_read_332))
            .method("read_333", self.op(Alg::jsonrpc_read_333))
            .method("read_334", self.op(Alg::jsonrpc_read_334))
            .method("read_335", self.op(Alg::jsonrpc_read_335))
            .method("read_336", self.op(Alg::jsonrpc_read_336))
            .method("read_337", self.op(Alg::jsonrpc_read_337))
            .method("read_338", self.op(Alg::jsonrpc_read_338))
            .method("read_339", self.op(Alg::jsonrpc_read_339))
            .method("read_340", self.op(Alg::jsonrpc_read_340))
            .method("read_341", self.op(Alg::jsonrpc_read_341))
            .method("read_342", self.op(Alg::jsonrpc_read_342))
            .method("read_343", self.op(Alg::jsonrpc_read_343))
            .method("read_344", self.op(Alg::jsonrpc_read_344))
            .method("read_345", self.op(Alg::jsonrpc_read_345))
            .method("read_346", self.op(Alg::jsonrpc_read_346))
            .method("read_347", self.op(Alg::jsonrpc_read_347))
            .method("read_348", self.op(Alg::jsonrpc_read_348))
            .method("read_349", self.op(Alg::jsonrpc_read_349))
            .method("read_350", self.op(Alg::jsonrpc_read_350))
            .method("read_351", self.op(Alg::jsonrpc_read_351))
            .method("read_352", self.op(Alg::jsonrpc_read_352))
            .method("read_353", self.op(Alg::jsonrpc_read_353))
            .method("read_354", self.op(Alg::jsonrpc_read_354))
            .method("read_355", self.op(Alg::jsonrpc_read_355))
            .method("read_356", self.op(Alg::jsonrpc_read_356))
            .method("read_357", self.op(Alg::jsonrpc_read_357))
            .method("read_358", self.op(Alg::jsonrpc_read_358))
            .method("read_359", self.op(Alg::jsonrpc_read_359))
            .method("read_360", self.op(Alg::jsonrpc_read_360))
            .method("read_361", self.op(Alg::jsonrpc_read_361))
            .method("read_362", self.op(Alg::jsonrpc_read_362))
            .method("read_363", self.op(Alg::jsonrpc_read_363))
            .method("read_364", self.op(Alg::jsonrpc_read_364))
            .method("read_365", self.op(Alg::jsonrpc_read_365))
            .method("read_366", self.op(Alg::jsonrpc_read_366))
            .method("read_367", self.op(Alg::jsonrpc_read_367))
            .method("read_368", self.op(Alg::jsonrpc_read_368))
            .method("read_369", self.op(Alg::jsonrpc_read_369))
            .method("read_370", self.op(Alg::jsonrpc_read_370))
            .method("read_371", self.op(Alg::jsonrpc_read_371))
            .method("read_372", self.op(Alg::jsonrpc_read_372))
            .method("read_373", self.op(Alg::jsonrpc_read_373))
            .method("read_374", self.op(Alg::jsonrpc_read_374))
            .method("read_375", self.op(Alg::jsonrpc_read_375))
            .method("read_376", self.op(Alg::jsonrpc_read_376))
            .method("read_377", self.op(Alg::jsonrpc_read_377))
            .method("read_378", self.op(Alg::jsonrpc_read_378))
            .method("read_379", self.op(Alg::jsonrpc_read_379))
            .method("read_380", self.op(Alg::jsonrpc_read_380))
            .method("read_381", self.op(Alg::jsonrpc_read_381))
            .method("read_382", self.op(Alg::jsonrpc_read_382))
            .method("read_383", self.op(Alg::jsonrpc_read_383))
            .method("read_384", self.op(Alg::jsonrpc_read_384))
            .method("read_385", self.op(Alg::jsonrpc_read_385))
            .method("read_386", self.op(Alg::jsonrpc_read_386))
            .method("read_387", self.op(Alg::jsonrpc_read_387))
            .method("read_388", self.op(Alg::jsonrpc_read_388))
            .method("read_389", self.op(Alg::jsonrpc_read_389))
            .method("read_390", self.op(Alg::jsonrpc_read_390))
            .method("read_391", self.op(Alg::jsonrpc_read_391))
            .method("read_392", self.op(Alg::jsonrpc_read_392))
            .method("read_393", self.op(Alg::jsonrpc_read_393))
            .method("read_394", self.op(Alg::jsonrpc_read_394))
            .method("read_395", self.op(Alg::jsonrpc_read_395))
            .method("read_396", self.op(Alg::jsonrpc_read_396))
            .method("read_397", self.op(Alg::jsonrpc_read_397))
            .method("read_398", self.op(Alg::jsonrpc_read_398))
            .method("read_399", self.op(Alg::jsonrpc_read_399))
            .method("read_400", self.op(Alg::jsonrpc_read_400))
            .method("read_401", self.op(Alg::jsonrpc_read_401))
            .method("read_402", self.op(Alg::jsonrpc_read_402))
            .method("read_403", self.op(Alg::jsonrpc_read_403))
            .method("read_404", self.op(Alg::jsonrpc_read_404))
            .method("read_405", self.op(Alg::jsonrpc_read_405))
            .method("read_406", self.op(Alg::jsonrpc_read_406))
            .method("read_407", self.op(Alg::jsonrpc_read_407))
            .method("read_408", self.op(Alg::jsonrpc_read_408))
            .method("read_409", self.op(Alg::jsonrpc_read_409))
            .method("read_410", self.op(Alg::jsonrpc_read_410))
            .method("read_411", self.op(Alg::jsonrpc_read_411))
            .method("read_412", self.op(Alg::jsonrpc_read_412))
            .method("read_413", self.op(Alg::jsonrpc_read_413))
            .method("read_414", self.op(Alg::jsonrpc_read_414))
            .method("read_415", self.op(Alg::jsonrpc_read_415))
            .method("read_416", self.op(Alg::jsonrpc_read_416))
            .method("read_417", self.op(Alg::jsonrpc_read_417))
            .method("read_418", self.op(Alg::jsonrpc_read_418))
            .method("read_419", self.op(Alg::jsonrpc_read_419))
            .method("read_420", self.op(Alg::jsonrpc_read_420))
            .method("read_421", self.op(Alg::jsonrpc_read_421))
            .method("read_422", self.op(Alg::jsonrpc_read_422))
            .method("read_423", self.op(Alg::jsonrpc_read_423))
            .method("read_424", self.op(Alg::jsonrpc_read_424))
            .method("read_425", self.op(Alg::jsonrpc_read_425))
            .method("read_426", self.op(Alg::jsonrpc_read_426))
            .method("read_427", self.op(Alg::jsonrpc_read_427))
            .method("read_428", self.op(Alg::jsonrpc_read_428))
            .method("read_429", self.op(Alg::jsonrpc_read_429))
            .method("read_430", self.op(Alg::jsonrpc_read_430))
            .method("read_431", self.op(Alg::jsonrpc_read_431))
            .method("read_432", self.op(Alg::jsonrpc_read_432))
            .method("read_433", self.op(Alg::jsonrpc_read_433))
            .method("read_434", self.op(Alg::jsonrpc_read_434))
            .method("read_435", self.op(Alg::jsonrpc_read_435))
            .method("read_436", self.op(Alg::jsonrpc_read_436))
            .method("read_437", self.op(Alg::jsonrpc_read_437))
            .method("read_438", self.op(Alg::jsonrpc_read_438))
            .method("read_439", self.op(Alg::jsonrpc_read_439))
            .method("read_440", self.op(Alg::jsonrpc_read_440))
            .method("read_441", self.op(Alg::jsonrpc_read_441))
            .method("read_442", self.op(Alg::jsonrpc_read_442))
            .method("read_443", self.op(Alg::jsonrpc_read_443))
            .method("read_444", self.op(Alg::jsonrpc_read_444))
            .method("read_445", self.op(Alg::jsonrpc_read_445))
            .method("read_446", self.op(Alg::jsonrpc_read_446))
            .method("read_447", self.op(Alg::jsonrpc_read_447))
            .method("read_448", self.op(Alg::jsonrpc_read_448))
            .method("read_449", self.op(Alg::jsonrpc_read_449))
            .method("read_450", self.op(Alg::jsonrpc_read_450))
            .method("read_451", self.op(Alg::jsonrpc_read_451))
            .method("read_452", self.op(Alg::jsonrpc_read_452))
            .method("read_453", self.op(Alg::jsonrpc_read_453))
            .method("read_454", self.op(Alg::jsonrpc_read_454))
            .method("read_455", self.op(Alg::jsonrpc_read_455))
            .method("read_456", self.op(Alg::jsonrpc_read_456))
            .method("read_457", self.op(Alg::jsonrpc_read_457))
            .method("read_458", self.op(Alg::jsonrpc_read_458))
            .method("read_459", self.op(Alg::jsonrpc_read_459))
            .method("read_460", self.op(Alg::jsonrpc_read_460))
            .method("read_461", self.op(Alg::jsonrpc_read_461))
            .method("read_462", self.op(Alg::jsonrpc_read_462))
            .method("read_463", self.op(Alg::jsonrpc_read_463))
            .method("read_464", self.op(Alg::jsonrpc_read_464))
            .method("read_465", self.op(Alg::jsonrpc_read_465))
            .method("read_466", self.op(Alg::jsonrpc_read_466))
            .method("read_467", self.op(Alg::jsonrpc_read_467))
            .method("read_468", self.op(Alg::jsonrpc_read_468))
            .method("read_469", self.op(Alg::jsonrpc_read_469))
            .method("read_470", self.op(Alg::jsonrpc_read_470))
            .method("read_471", self.op(Alg::jsonrpc_read_471))
            .method("read_472", self.op(Alg::jsonrpc_read_472))
            .method("read_473", self.op(Alg::jsonrpc_read_473))
            .method("read_474", self.op(Alg::jsonrpc_read_474))
            .method("read_475", self.op(Alg::jsonrpc_read_475))
            .method("read_476", self.op(Alg::jsonrpc_read_476))
            .method("read_477", self.op(Alg::jsonrpc_read_477))
            .method("read_478", self.op(Alg::jsonrpc_read_478))
            .method("read_479", self.op(Alg::jsonrpc_read_479))
            .method("read_480", self.op(Alg::jsonrpc_read_480))
            .method("read_481", self.op(Alg::jsonrpc_read_481))
            .method("read_482", self.op(Alg::jsonrpc_read_482))
            .method("read_483", self.op(Alg::jsonrpc_read_483))
            .method("read_484", self.op(Alg::jsonrpc_read_484))
            .method("read_485", self.op(Alg::jsonrpc_read_485))
            .method("read_486", self.op(Alg::jsonrpc_read_486))
            .method("read_487", self.op(Alg::jsonrpc_read_487))
            .method("read_488", self.op(Alg::jsonrpc_read_488))
            .method("read_489", self.op(Alg::jsonrpc_read_489))
            .method("read_490", self.op(Alg::jsonrpc_read_490))
            .method("read_491", self.op(Alg::jsonrpc_read_491))
            .method("read_492", self.op(Alg::jsonrpc_read_492))
            .method("read_493", self.op(Alg::jsonrpc_read_493))
            .method("read_494", self.op(Alg::jsonrpc_read_494))
            .method("read_495", self.op(Alg::jsonrpc_read_495))
            .method("read_496", self.op(Alg::jsonrpc_read_496))
            .method("read_497", self.op(Alg::jsonrpc_read_497))
            .method("read_498", self.op(Alg::jsonrpc_read_498))
            .method("read_499", self.op(Alg::jsonrpc_read_499))
    }
}

/// Names a domain for the declaration to be read against.
struct Domain;

impl ReadingAlg for Domain {
    type Reading = u32;

    async fn read(&self, value: u32) -> Self::Reading {
        value
    }
}

#[test]
fn a_declaration_of_many_methods_states_one_program() {
    let _program = Domain.many_rpc::<Domain>();
}
