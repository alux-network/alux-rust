//! One declaration states 500 routes, and stating them costs what stating one of them costs.
//!
//! A declaration read as a nested type costs the square of its route count, or worse. The
//! declaration is compiled to one route per endpoint instead, so what this guards is that checking
//! it stays ordinary work. The assertion is the compile; the test body only names the program the
//! declaration denotes.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_http::{HttpApiAlg, JsonOutAlg, http};
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
    async fn http_read_0(&self) -> This::Reading {
        self.read(0).await
    }

    /// Reads the value at 1.
    async fn http_read_1(&self) -> This::Reading {
        self.read(1).await
    }

    /// Reads the value at 2.
    async fn http_read_2(&self) -> This::Reading {
        self.read(2).await
    }

    /// Reads the value at 3.
    async fn http_read_3(&self) -> This::Reading {
        self.read(3).await
    }

    /// Reads the value at 4.
    async fn http_read_4(&self) -> This::Reading {
        self.read(4).await
    }

    /// Reads the value at 5.
    async fn http_read_5(&self) -> This::Reading {
        self.read(5).await
    }

    /// Reads the value at 6.
    async fn http_read_6(&self) -> This::Reading {
        self.read(6).await
    }

    /// Reads the value at 7.
    async fn http_read_7(&self) -> This::Reading {
        self.read(7).await
    }

    /// Reads the value at 8.
    async fn http_read_8(&self) -> This::Reading {
        self.read(8).await
    }

    /// Reads the value at 9.
    async fn http_read_9(&self) -> This::Reading {
        self.read(9).await
    }

    /// Reads the value at 10.
    async fn http_read_10(&self) -> This::Reading {
        self.read(10).await
    }

    /// Reads the value at 11.
    async fn http_read_11(&self) -> This::Reading {
        self.read(11).await
    }

    /// Reads the value at 12.
    async fn http_read_12(&self) -> This::Reading {
        self.read(12).await
    }

    /// Reads the value at 13.
    async fn http_read_13(&self) -> This::Reading {
        self.read(13).await
    }

    /// Reads the value at 14.
    async fn http_read_14(&self) -> This::Reading {
        self.read(14).await
    }

    /// Reads the value at 15.
    async fn http_read_15(&self) -> This::Reading {
        self.read(15).await
    }

    /// Reads the value at 16.
    async fn http_read_16(&self) -> This::Reading {
        self.read(16).await
    }

    /// Reads the value at 17.
    async fn http_read_17(&self) -> This::Reading {
        self.read(17).await
    }

    /// Reads the value at 18.
    async fn http_read_18(&self) -> This::Reading {
        self.read(18).await
    }

    /// Reads the value at 19.
    async fn http_read_19(&self) -> This::Reading {
        self.read(19).await
    }

    /// Reads the value at 20.
    async fn http_read_20(&self) -> This::Reading {
        self.read(20).await
    }

    /// Reads the value at 21.
    async fn http_read_21(&self) -> This::Reading {
        self.read(21).await
    }

    /// Reads the value at 22.
    async fn http_read_22(&self) -> This::Reading {
        self.read(22).await
    }

    /// Reads the value at 23.
    async fn http_read_23(&self) -> This::Reading {
        self.read(23).await
    }

    /// Reads the value at 24.
    async fn http_read_24(&self) -> This::Reading {
        self.read(24).await
    }

    /// Reads the value at 25.
    async fn http_read_25(&self) -> This::Reading {
        self.read(25).await
    }

    /// Reads the value at 26.
    async fn http_read_26(&self) -> This::Reading {
        self.read(26).await
    }

    /// Reads the value at 27.
    async fn http_read_27(&self) -> This::Reading {
        self.read(27).await
    }

    /// Reads the value at 28.
    async fn http_read_28(&self) -> This::Reading {
        self.read(28).await
    }

    /// Reads the value at 29.
    async fn http_read_29(&self) -> This::Reading {
        self.read(29).await
    }

    /// Reads the value at 30.
    async fn http_read_30(&self) -> This::Reading {
        self.read(30).await
    }

    /// Reads the value at 31.
    async fn http_read_31(&self) -> This::Reading {
        self.read(31).await
    }

    /// Reads the value at 32.
    async fn http_read_32(&self) -> This::Reading {
        self.read(32).await
    }

    /// Reads the value at 33.
    async fn http_read_33(&self) -> This::Reading {
        self.read(33).await
    }

    /// Reads the value at 34.
    async fn http_read_34(&self) -> This::Reading {
        self.read(34).await
    }

    /// Reads the value at 35.
    async fn http_read_35(&self) -> This::Reading {
        self.read(35).await
    }

    /// Reads the value at 36.
    async fn http_read_36(&self) -> This::Reading {
        self.read(36).await
    }

    /// Reads the value at 37.
    async fn http_read_37(&self) -> This::Reading {
        self.read(37).await
    }

    /// Reads the value at 38.
    async fn http_read_38(&self) -> This::Reading {
        self.read(38).await
    }

    /// Reads the value at 39.
    async fn http_read_39(&self) -> This::Reading {
        self.read(39).await
    }

    /// Reads the value at 40.
    async fn http_read_40(&self) -> This::Reading {
        self.read(40).await
    }

    /// Reads the value at 41.
    async fn http_read_41(&self) -> This::Reading {
        self.read(41).await
    }

    /// Reads the value at 42.
    async fn http_read_42(&self) -> This::Reading {
        self.read(42).await
    }

    /// Reads the value at 43.
    async fn http_read_43(&self) -> This::Reading {
        self.read(43).await
    }

    /// Reads the value at 44.
    async fn http_read_44(&self) -> This::Reading {
        self.read(44).await
    }

    /// Reads the value at 45.
    async fn http_read_45(&self) -> This::Reading {
        self.read(45).await
    }

    /// Reads the value at 46.
    async fn http_read_46(&self) -> This::Reading {
        self.read(46).await
    }

    /// Reads the value at 47.
    async fn http_read_47(&self) -> This::Reading {
        self.read(47).await
    }

    /// Reads the value at 48.
    async fn http_read_48(&self) -> This::Reading {
        self.read(48).await
    }

    /// Reads the value at 49.
    async fn http_read_49(&self) -> This::Reading {
        self.read(49).await
    }

    /// Reads the value at 50.
    async fn http_read_50(&self) -> This::Reading {
        self.read(50).await
    }

    /// Reads the value at 51.
    async fn http_read_51(&self) -> This::Reading {
        self.read(51).await
    }

    /// Reads the value at 52.
    async fn http_read_52(&self) -> This::Reading {
        self.read(52).await
    }

    /// Reads the value at 53.
    async fn http_read_53(&self) -> This::Reading {
        self.read(53).await
    }

    /// Reads the value at 54.
    async fn http_read_54(&self) -> This::Reading {
        self.read(54).await
    }

    /// Reads the value at 55.
    async fn http_read_55(&self) -> This::Reading {
        self.read(55).await
    }

    /// Reads the value at 56.
    async fn http_read_56(&self) -> This::Reading {
        self.read(56).await
    }

    /// Reads the value at 57.
    async fn http_read_57(&self) -> This::Reading {
        self.read(57).await
    }

    /// Reads the value at 58.
    async fn http_read_58(&self) -> This::Reading {
        self.read(58).await
    }

    /// Reads the value at 59.
    async fn http_read_59(&self) -> This::Reading {
        self.read(59).await
    }

    /// Reads the value at 60.
    async fn http_read_60(&self) -> This::Reading {
        self.read(60).await
    }

    /// Reads the value at 61.
    async fn http_read_61(&self) -> This::Reading {
        self.read(61).await
    }

    /// Reads the value at 62.
    async fn http_read_62(&self) -> This::Reading {
        self.read(62).await
    }

    /// Reads the value at 63.
    async fn http_read_63(&self) -> This::Reading {
        self.read(63).await
    }

    /// Reads the value at 64.
    async fn http_read_64(&self) -> This::Reading {
        self.read(64).await
    }

    /// Reads the value at 65.
    async fn http_read_65(&self) -> This::Reading {
        self.read(65).await
    }

    /// Reads the value at 66.
    async fn http_read_66(&self) -> This::Reading {
        self.read(66).await
    }

    /// Reads the value at 67.
    async fn http_read_67(&self) -> This::Reading {
        self.read(67).await
    }

    /// Reads the value at 68.
    async fn http_read_68(&self) -> This::Reading {
        self.read(68).await
    }

    /// Reads the value at 69.
    async fn http_read_69(&self) -> This::Reading {
        self.read(69).await
    }

    /// Reads the value at 70.
    async fn http_read_70(&self) -> This::Reading {
        self.read(70).await
    }

    /// Reads the value at 71.
    async fn http_read_71(&self) -> This::Reading {
        self.read(71).await
    }

    /// Reads the value at 72.
    async fn http_read_72(&self) -> This::Reading {
        self.read(72).await
    }

    /// Reads the value at 73.
    async fn http_read_73(&self) -> This::Reading {
        self.read(73).await
    }

    /// Reads the value at 74.
    async fn http_read_74(&self) -> This::Reading {
        self.read(74).await
    }

    /// Reads the value at 75.
    async fn http_read_75(&self) -> This::Reading {
        self.read(75).await
    }

    /// Reads the value at 76.
    async fn http_read_76(&self) -> This::Reading {
        self.read(76).await
    }

    /// Reads the value at 77.
    async fn http_read_77(&self) -> This::Reading {
        self.read(77).await
    }

    /// Reads the value at 78.
    async fn http_read_78(&self) -> This::Reading {
        self.read(78).await
    }

    /// Reads the value at 79.
    async fn http_read_79(&self) -> This::Reading {
        self.read(79).await
    }

    /// Reads the value at 80.
    async fn http_read_80(&self) -> This::Reading {
        self.read(80).await
    }

    /// Reads the value at 81.
    async fn http_read_81(&self) -> This::Reading {
        self.read(81).await
    }

    /// Reads the value at 82.
    async fn http_read_82(&self) -> This::Reading {
        self.read(82).await
    }

    /// Reads the value at 83.
    async fn http_read_83(&self) -> This::Reading {
        self.read(83).await
    }

    /// Reads the value at 84.
    async fn http_read_84(&self) -> This::Reading {
        self.read(84).await
    }

    /// Reads the value at 85.
    async fn http_read_85(&self) -> This::Reading {
        self.read(85).await
    }

    /// Reads the value at 86.
    async fn http_read_86(&self) -> This::Reading {
        self.read(86).await
    }

    /// Reads the value at 87.
    async fn http_read_87(&self) -> This::Reading {
        self.read(87).await
    }

    /// Reads the value at 88.
    async fn http_read_88(&self) -> This::Reading {
        self.read(88).await
    }

    /// Reads the value at 89.
    async fn http_read_89(&self) -> This::Reading {
        self.read(89).await
    }

    /// Reads the value at 90.
    async fn http_read_90(&self) -> This::Reading {
        self.read(90).await
    }

    /// Reads the value at 91.
    async fn http_read_91(&self) -> This::Reading {
        self.read(91).await
    }

    /// Reads the value at 92.
    async fn http_read_92(&self) -> This::Reading {
        self.read(92).await
    }

    /// Reads the value at 93.
    async fn http_read_93(&self) -> This::Reading {
        self.read(93).await
    }

    /// Reads the value at 94.
    async fn http_read_94(&self) -> This::Reading {
        self.read(94).await
    }

    /// Reads the value at 95.
    async fn http_read_95(&self) -> This::Reading {
        self.read(95).await
    }

    /// Reads the value at 96.
    async fn http_read_96(&self) -> This::Reading {
        self.read(96).await
    }

    /// Reads the value at 97.
    async fn http_read_97(&self) -> This::Reading {
        self.read(97).await
    }

    /// Reads the value at 98.
    async fn http_read_98(&self) -> This::Reading {
        self.read(98).await
    }

    /// Reads the value at 99.
    async fn http_read_99(&self) -> This::Reading {
        self.read(99).await
    }

    /// Reads the value at 100.
    async fn http_read_100(&self) -> This::Reading {
        self.read(100).await
    }

    /// Reads the value at 101.
    async fn http_read_101(&self) -> This::Reading {
        self.read(101).await
    }

    /// Reads the value at 102.
    async fn http_read_102(&self) -> This::Reading {
        self.read(102).await
    }

    /// Reads the value at 103.
    async fn http_read_103(&self) -> This::Reading {
        self.read(103).await
    }

    /// Reads the value at 104.
    async fn http_read_104(&self) -> This::Reading {
        self.read(104).await
    }

    /// Reads the value at 105.
    async fn http_read_105(&self) -> This::Reading {
        self.read(105).await
    }

    /// Reads the value at 106.
    async fn http_read_106(&self) -> This::Reading {
        self.read(106).await
    }

    /// Reads the value at 107.
    async fn http_read_107(&self) -> This::Reading {
        self.read(107).await
    }

    /// Reads the value at 108.
    async fn http_read_108(&self) -> This::Reading {
        self.read(108).await
    }

    /// Reads the value at 109.
    async fn http_read_109(&self) -> This::Reading {
        self.read(109).await
    }

    /// Reads the value at 110.
    async fn http_read_110(&self) -> This::Reading {
        self.read(110).await
    }

    /// Reads the value at 111.
    async fn http_read_111(&self) -> This::Reading {
        self.read(111).await
    }

    /// Reads the value at 112.
    async fn http_read_112(&self) -> This::Reading {
        self.read(112).await
    }

    /// Reads the value at 113.
    async fn http_read_113(&self) -> This::Reading {
        self.read(113).await
    }

    /// Reads the value at 114.
    async fn http_read_114(&self) -> This::Reading {
        self.read(114).await
    }

    /// Reads the value at 115.
    async fn http_read_115(&self) -> This::Reading {
        self.read(115).await
    }

    /// Reads the value at 116.
    async fn http_read_116(&self) -> This::Reading {
        self.read(116).await
    }

    /// Reads the value at 117.
    async fn http_read_117(&self) -> This::Reading {
        self.read(117).await
    }

    /// Reads the value at 118.
    async fn http_read_118(&self) -> This::Reading {
        self.read(118).await
    }

    /// Reads the value at 119.
    async fn http_read_119(&self) -> This::Reading {
        self.read(119).await
    }

    /// Reads the value at 120.
    async fn http_read_120(&self) -> This::Reading {
        self.read(120).await
    }

    /// Reads the value at 121.
    async fn http_read_121(&self) -> This::Reading {
        self.read(121).await
    }

    /// Reads the value at 122.
    async fn http_read_122(&self) -> This::Reading {
        self.read(122).await
    }

    /// Reads the value at 123.
    async fn http_read_123(&self) -> This::Reading {
        self.read(123).await
    }

    /// Reads the value at 124.
    async fn http_read_124(&self) -> This::Reading {
        self.read(124).await
    }

    /// Reads the value at 125.
    async fn http_read_125(&self) -> This::Reading {
        self.read(125).await
    }

    /// Reads the value at 126.
    async fn http_read_126(&self) -> This::Reading {
        self.read(126).await
    }

    /// Reads the value at 127.
    async fn http_read_127(&self) -> This::Reading {
        self.read(127).await
    }

    /// Reads the value at 128.
    async fn http_read_128(&self) -> This::Reading {
        self.read(128).await
    }

    /// Reads the value at 129.
    async fn http_read_129(&self) -> This::Reading {
        self.read(129).await
    }

    /// Reads the value at 130.
    async fn http_read_130(&self) -> This::Reading {
        self.read(130).await
    }

    /// Reads the value at 131.
    async fn http_read_131(&self) -> This::Reading {
        self.read(131).await
    }

    /// Reads the value at 132.
    async fn http_read_132(&self) -> This::Reading {
        self.read(132).await
    }

    /// Reads the value at 133.
    async fn http_read_133(&self) -> This::Reading {
        self.read(133).await
    }

    /// Reads the value at 134.
    async fn http_read_134(&self) -> This::Reading {
        self.read(134).await
    }

    /// Reads the value at 135.
    async fn http_read_135(&self) -> This::Reading {
        self.read(135).await
    }

    /// Reads the value at 136.
    async fn http_read_136(&self) -> This::Reading {
        self.read(136).await
    }

    /// Reads the value at 137.
    async fn http_read_137(&self) -> This::Reading {
        self.read(137).await
    }

    /// Reads the value at 138.
    async fn http_read_138(&self) -> This::Reading {
        self.read(138).await
    }

    /// Reads the value at 139.
    async fn http_read_139(&self) -> This::Reading {
        self.read(139).await
    }

    /// Reads the value at 140.
    async fn http_read_140(&self) -> This::Reading {
        self.read(140).await
    }

    /// Reads the value at 141.
    async fn http_read_141(&self) -> This::Reading {
        self.read(141).await
    }

    /// Reads the value at 142.
    async fn http_read_142(&self) -> This::Reading {
        self.read(142).await
    }

    /// Reads the value at 143.
    async fn http_read_143(&self) -> This::Reading {
        self.read(143).await
    }

    /// Reads the value at 144.
    async fn http_read_144(&self) -> This::Reading {
        self.read(144).await
    }

    /// Reads the value at 145.
    async fn http_read_145(&self) -> This::Reading {
        self.read(145).await
    }

    /// Reads the value at 146.
    async fn http_read_146(&self) -> This::Reading {
        self.read(146).await
    }

    /// Reads the value at 147.
    async fn http_read_147(&self) -> This::Reading {
        self.read(147).await
    }

    /// Reads the value at 148.
    async fn http_read_148(&self) -> This::Reading {
        self.read(148).await
    }

    /// Reads the value at 149.
    async fn http_read_149(&self) -> This::Reading {
        self.read(149).await
    }

    /// Reads the value at 150.
    async fn http_read_150(&self) -> This::Reading {
        self.read(150).await
    }

    /// Reads the value at 151.
    async fn http_read_151(&self) -> This::Reading {
        self.read(151).await
    }

    /// Reads the value at 152.
    async fn http_read_152(&self) -> This::Reading {
        self.read(152).await
    }

    /// Reads the value at 153.
    async fn http_read_153(&self) -> This::Reading {
        self.read(153).await
    }

    /// Reads the value at 154.
    async fn http_read_154(&self) -> This::Reading {
        self.read(154).await
    }

    /// Reads the value at 155.
    async fn http_read_155(&self) -> This::Reading {
        self.read(155).await
    }

    /// Reads the value at 156.
    async fn http_read_156(&self) -> This::Reading {
        self.read(156).await
    }

    /// Reads the value at 157.
    async fn http_read_157(&self) -> This::Reading {
        self.read(157).await
    }

    /// Reads the value at 158.
    async fn http_read_158(&self) -> This::Reading {
        self.read(158).await
    }

    /// Reads the value at 159.
    async fn http_read_159(&self) -> This::Reading {
        self.read(159).await
    }

    /// Reads the value at 160.
    async fn http_read_160(&self) -> This::Reading {
        self.read(160).await
    }

    /// Reads the value at 161.
    async fn http_read_161(&self) -> This::Reading {
        self.read(161).await
    }

    /// Reads the value at 162.
    async fn http_read_162(&self) -> This::Reading {
        self.read(162).await
    }

    /// Reads the value at 163.
    async fn http_read_163(&self) -> This::Reading {
        self.read(163).await
    }

    /// Reads the value at 164.
    async fn http_read_164(&self) -> This::Reading {
        self.read(164).await
    }

    /// Reads the value at 165.
    async fn http_read_165(&self) -> This::Reading {
        self.read(165).await
    }

    /// Reads the value at 166.
    async fn http_read_166(&self) -> This::Reading {
        self.read(166).await
    }

    /// Reads the value at 167.
    async fn http_read_167(&self) -> This::Reading {
        self.read(167).await
    }

    /// Reads the value at 168.
    async fn http_read_168(&self) -> This::Reading {
        self.read(168).await
    }

    /// Reads the value at 169.
    async fn http_read_169(&self) -> This::Reading {
        self.read(169).await
    }

    /// Reads the value at 170.
    async fn http_read_170(&self) -> This::Reading {
        self.read(170).await
    }

    /// Reads the value at 171.
    async fn http_read_171(&self) -> This::Reading {
        self.read(171).await
    }

    /// Reads the value at 172.
    async fn http_read_172(&self) -> This::Reading {
        self.read(172).await
    }

    /// Reads the value at 173.
    async fn http_read_173(&self) -> This::Reading {
        self.read(173).await
    }

    /// Reads the value at 174.
    async fn http_read_174(&self) -> This::Reading {
        self.read(174).await
    }

    /// Reads the value at 175.
    async fn http_read_175(&self) -> This::Reading {
        self.read(175).await
    }

    /// Reads the value at 176.
    async fn http_read_176(&self) -> This::Reading {
        self.read(176).await
    }

    /// Reads the value at 177.
    async fn http_read_177(&self) -> This::Reading {
        self.read(177).await
    }

    /// Reads the value at 178.
    async fn http_read_178(&self) -> This::Reading {
        self.read(178).await
    }

    /// Reads the value at 179.
    async fn http_read_179(&self) -> This::Reading {
        self.read(179).await
    }

    /// Reads the value at 180.
    async fn http_read_180(&self) -> This::Reading {
        self.read(180).await
    }

    /// Reads the value at 181.
    async fn http_read_181(&self) -> This::Reading {
        self.read(181).await
    }

    /// Reads the value at 182.
    async fn http_read_182(&self) -> This::Reading {
        self.read(182).await
    }

    /// Reads the value at 183.
    async fn http_read_183(&self) -> This::Reading {
        self.read(183).await
    }

    /// Reads the value at 184.
    async fn http_read_184(&self) -> This::Reading {
        self.read(184).await
    }

    /// Reads the value at 185.
    async fn http_read_185(&self) -> This::Reading {
        self.read(185).await
    }

    /// Reads the value at 186.
    async fn http_read_186(&self) -> This::Reading {
        self.read(186).await
    }

    /// Reads the value at 187.
    async fn http_read_187(&self) -> This::Reading {
        self.read(187).await
    }

    /// Reads the value at 188.
    async fn http_read_188(&self) -> This::Reading {
        self.read(188).await
    }

    /// Reads the value at 189.
    async fn http_read_189(&self) -> This::Reading {
        self.read(189).await
    }

    /// Reads the value at 190.
    async fn http_read_190(&self) -> This::Reading {
        self.read(190).await
    }

    /// Reads the value at 191.
    async fn http_read_191(&self) -> This::Reading {
        self.read(191).await
    }

    /// Reads the value at 192.
    async fn http_read_192(&self) -> This::Reading {
        self.read(192).await
    }

    /// Reads the value at 193.
    async fn http_read_193(&self) -> This::Reading {
        self.read(193).await
    }

    /// Reads the value at 194.
    async fn http_read_194(&self) -> This::Reading {
        self.read(194).await
    }

    /// Reads the value at 195.
    async fn http_read_195(&self) -> This::Reading {
        self.read(195).await
    }

    /// Reads the value at 196.
    async fn http_read_196(&self) -> This::Reading {
        self.read(196).await
    }

    /// Reads the value at 197.
    async fn http_read_197(&self) -> This::Reading {
        self.read(197).await
    }

    /// Reads the value at 198.
    async fn http_read_198(&self) -> This::Reading {
        self.read(198).await
    }

    /// Reads the value at 199.
    async fn http_read_199(&self) -> This::Reading {
        self.read(199).await
    }

    /// Reads the value at 200.
    async fn http_read_200(&self) -> This::Reading {
        self.read(200).await
    }

    /// Reads the value at 201.
    async fn http_read_201(&self) -> This::Reading {
        self.read(201).await
    }

    /// Reads the value at 202.
    async fn http_read_202(&self) -> This::Reading {
        self.read(202).await
    }

    /// Reads the value at 203.
    async fn http_read_203(&self) -> This::Reading {
        self.read(203).await
    }

    /// Reads the value at 204.
    async fn http_read_204(&self) -> This::Reading {
        self.read(204).await
    }

    /// Reads the value at 205.
    async fn http_read_205(&self) -> This::Reading {
        self.read(205).await
    }

    /// Reads the value at 206.
    async fn http_read_206(&self) -> This::Reading {
        self.read(206).await
    }

    /// Reads the value at 207.
    async fn http_read_207(&self) -> This::Reading {
        self.read(207).await
    }

    /// Reads the value at 208.
    async fn http_read_208(&self) -> This::Reading {
        self.read(208).await
    }

    /// Reads the value at 209.
    async fn http_read_209(&self) -> This::Reading {
        self.read(209).await
    }

    /// Reads the value at 210.
    async fn http_read_210(&self) -> This::Reading {
        self.read(210).await
    }

    /// Reads the value at 211.
    async fn http_read_211(&self) -> This::Reading {
        self.read(211).await
    }

    /// Reads the value at 212.
    async fn http_read_212(&self) -> This::Reading {
        self.read(212).await
    }

    /// Reads the value at 213.
    async fn http_read_213(&self) -> This::Reading {
        self.read(213).await
    }

    /// Reads the value at 214.
    async fn http_read_214(&self) -> This::Reading {
        self.read(214).await
    }

    /// Reads the value at 215.
    async fn http_read_215(&self) -> This::Reading {
        self.read(215).await
    }

    /// Reads the value at 216.
    async fn http_read_216(&self) -> This::Reading {
        self.read(216).await
    }

    /// Reads the value at 217.
    async fn http_read_217(&self) -> This::Reading {
        self.read(217).await
    }

    /// Reads the value at 218.
    async fn http_read_218(&self) -> This::Reading {
        self.read(218).await
    }

    /// Reads the value at 219.
    async fn http_read_219(&self) -> This::Reading {
        self.read(219).await
    }

    /// Reads the value at 220.
    async fn http_read_220(&self) -> This::Reading {
        self.read(220).await
    }

    /// Reads the value at 221.
    async fn http_read_221(&self) -> This::Reading {
        self.read(221).await
    }

    /// Reads the value at 222.
    async fn http_read_222(&self) -> This::Reading {
        self.read(222).await
    }

    /// Reads the value at 223.
    async fn http_read_223(&self) -> This::Reading {
        self.read(223).await
    }

    /// Reads the value at 224.
    async fn http_read_224(&self) -> This::Reading {
        self.read(224).await
    }

    /// Reads the value at 225.
    async fn http_read_225(&self) -> This::Reading {
        self.read(225).await
    }

    /// Reads the value at 226.
    async fn http_read_226(&self) -> This::Reading {
        self.read(226).await
    }

    /// Reads the value at 227.
    async fn http_read_227(&self) -> This::Reading {
        self.read(227).await
    }

    /// Reads the value at 228.
    async fn http_read_228(&self) -> This::Reading {
        self.read(228).await
    }

    /// Reads the value at 229.
    async fn http_read_229(&self) -> This::Reading {
        self.read(229).await
    }

    /// Reads the value at 230.
    async fn http_read_230(&self) -> This::Reading {
        self.read(230).await
    }

    /// Reads the value at 231.
    async fn http_read_231(&self) -> This::Reading {
        self.read(231).await
    }

    /// Reads the value at 232.
    async fn http_read_232(&self) -> This::Reading {
        self.read(232).await
    }

    /// Reads the value at 233.
    async fn http_read_233(&self) -> This::Reading {
        self.read(233).await
    }

    /// Reads the value at 234.
    async fn http_read_234(&self) -> This::Reading {
        self.read(234).await
    }

    /// Reads the value at 235.
    async fn http_read_235(&self) -> This::Reading {
        self.read(235).await
    }

    /// Reads the value at 236.
    async fn http_read_236(&self) -> This::Reading {
        self.read(236).await
    }

    /// Reads the value at 237.
    async fn http_read_237(&self) -> This::Reading {
        self.read(237).await
    }

    /// Reads the value at 238.
    async fn http_read_238(&self) -> This::Reading {
        self.read(238).await
    }

    /// Reads the value at 239.
    async fn http_read_239(&self) -> This::Reading {
        self.read(239).await
    }

    /// Reads the value at 240.
    async fn http_read_240(&self) -> This::Reading {
        self.read(240).await
    }

    /// Reads the value at 241.
    async fn http_read_241(&self) -> This::Reading {
        self.read(241).await
    }

    /// Reads the value at 242.
    async fn http_read_242(&self) -> This::Reading {
        self.read(242).await
    }

    /// Reads the value at 243.
    async fn http_read_243(&self) -> This::Reading {
        self.read(243).await
    }

    /// Reads the value at 244.
    async fn http_read_244(&self) -> This::Reading {
        self.read(244).await
    }

    /// Reads the value at 245.
    async fn http_read_245(&self) -> This::Reading {
        self.read(245).await
    }

    /// Reads the value at 246.
    async fn http_read_246(&self) -> This::Reading {
        self.read(246).await
    }

    /// Reads the value at 247.
    async fn http_read_247(&self) -> This::Reading {
        self.read(247).await
    }

    /// Reads the value at 248.
    async fn http_read_248(&self) -> This::Reading {
        self.read(248).await
    }

    /// Reads the value at 249.
    async fn http_read_249(&self) -> This::Reading {
        self.read(249).await
    }

    /// Reads the value at 250.
    async fn http_read_250(&self) -> This::Reading {
        self.read(250).await
    }

    /// Reads the value at 251.
    async fn http_read_251(&self) -> This::Reading {
        self.read(251).await
    }

    /// Reads the value at 252.
    async fn http_read_252(&self) -> This::Reading {
        self.read(252).await
    }

    /// Reads the value at 253.
    async fn http_read_253(&self) -> This::Reading {
        self.read(253).await
    }

    /// Reads the value at 254.
    async fn http_read_254(&self) -> This::Reading {
        self.read(254).await
    }

    /// Reads the value at 255.
    async fn http_read_255(&self) -> This::Reading {
        self.read(255).await
    }

    /// Reads the value at 256.
    async fn http_read_256(&self) -> This::Reading {
        self.read(256).await
    }

    /// Reads the value at 257.
    async fn http_read_257(&self) -> This::Reading {
        self.read(257).await
    }

    /// Reads the value at 258.
    async fn http_read_258(&self) -> This::Reading {
        self.read(258).await
    }

    /// Reads the value at 259.
    async fn http_read_259(&self) -> This::Reading {
        self.read(259).await
    }

    /// Reads the value at 260.
    async fn http_read_260(&self) -> This::Reading {
        self.read(260).await
    }

    /// Reads the value at 261.
    async fn http_read_261(&self) -> This::Reading {
        self.read(261).await
    }

    /// Reads the value at 262.
    async fn http_read_262(&self) -> This::Reading {
        self.read(262).await
    }

    /// Reads the value at 263.
    async fn http_read_263(&self) -> This::Reading {
        self.read(263).await
    }

    /// Reads the value at 264.
    async fn http_read_264(&self) -> This::Reading {
        self.read(264).await
    }

    /// Reads the value at 265.
    async fn http_read_265(&self) -> This::Reading {
        self.read(265).await
    }

    /// Reads the value at 266.
    async fn http_read_266(&self) -> This::Reading {
        self.read(266).await
    }

    /// Reads the value at 267.
    async fn http_read_267(&self) -> This::Reading {
        self.read(267).await
    }

    /// Reads the value at 268.
    async fn http_read_268(&self) -> This::Reading {
        self.read(268).await
    }

    /// Reads the value at 269.
    async fn http_read_269(&self) -> This::Reading {
        self.read(269).await
    }

    /// Reads the value at 270.
    async fn http_read_270(&self) -> This::Reading {
        self.read(270).await
    }

    /// Reads the value at 271.
    async fn http_read_271(&self) -> This::Reading {
        self.read(271).await
    }

    /// Reads the value at 272.
    async fn http_read_272(&self) -> This::Reading {
        self.read(272).await
    }

    /// Reads the value at 273.
    async fn http_read_273(&self) -> This::Reading {
        self.read(273).await
    }

    /// Reads the value at 274.
    async fn http_read_274(&self) -> This::Reading {
        self.read(274).await
    }

    /// Reads the value at 275.
    async fn http_read_275(&self) -> This::Reading {
        self.read(275).await
    }

    /// Reads the value at 276.
    async fn http_read_276(&self) -> This::Reading {
        self.read(276).await
    }

    /// Reads the value at 277.
    async fn http_read_277(&self) -> This::Reading {
        self.read(277).await
    }

    /// Reads the value at 278.
    async fn http_read_278(&self) -> This::Reading {
        self.read(278).await
    }

    /// Reads the value at 279.
    async fn http_read_279(&self) -> This::Reading {
        self.read(279).await
    }

    /// Reads the value at 280.
    async fn http_read_280(&self) -> This::Reading {
        self.read(280).await
    }

    /// Reads the value at 281.
    async fn http_read_281(&self) -> This::Reading {
        self.read(281).await
    }

    /// Reads the value at 282.
    async fn http_read_282(&self) -> This::Reading {
        self.read(282).await
    }

    /// Reads the value at 283.
    async fn http_read_283(&self) -> This::Reading {
        self.read(283).await
    }

    /// Reads the value at 284.
    async fn http_read_284(&self) -> This::Reading {
        self.read(284).await
    }

    /// Reads the value at 285.
    async fn http_read_285(&self) -> This::Reading {
        self.read(285).await
    }

    /// Reads the value at 286.
    async fn http_read_286(&self) -> This::Reading {
        self.read(286).await
    }

    /// Reads the value at 287.
    async fn http_read_287(&self) -> This::Reading {
        self.read(287).await
    }

    /// Reads the value at 288.
    async fn http_read_288(&self) -> This::Reading {
        self.read(288).await
    }

    /// Reads the value at 289.
    async fn http_read_289(&self) -> This::Reading {
        self.read(289).await
    }

    /// Reads the value at 290.
    async fn http_read_290(&self) -> This::Reading {
        self.read(290).await
    }

    /// Reads the value at 291.
    async fn http_read_291(&self) -> This::Reading {
        self.read(291).await
    }

    /// Reads the value at 292.
    async fn http_read_292(&self) -> This::Reading {
        self.read(292).await
    }

    /// Reads the value at 293.
    async fn http_read_293(&self) -> This::Reading {
        self.read(293).await
    }

    /// Reads the value at 294.
    async fn http_read_294(&self) -> This::Reading {
        self.read(294).await
    }

    /// Reads the value at 295.
    async fn http_read_295(&self) -> This::Reading {
        self.read(295).await
    }

    /// Reads the value at 296.
    async fn http_read_296(&self) -> This::Reading {
        self.read(296).await
    }

    /// Reads the value at 297.
    async fn http_read_297(&self) -> This::Reading {
        self.read(297).await
    }

    /// Reads the value at 298.
    async fn http_read_298(&self) -> This::Reading {
        self.read(298).await
    }

    /// Reads the value at 299.
    async fn http_read_299(&self) -> This::Reading {
        self.read(299).await
    }

    /// Reads the value at 300.
    async fn http_read_300(&self) -> This::Reading {
        self.read(300).await
    }

    /// Reads the value at 301.
    async fn http_read_301(&self) -> This::Reading {
        self.read(301).await
    }

    /// Reads the value at 302.
    async fn http_read_302(&self) -> This::Reading {
        self.read(302).await
    }

    /// Reads the value at 303.
    async fn http_read_303(&self) -> This::Reading {
        self.read(303).await
    }

    /// Reads the value at 304.
    async fn http_read_304(&self) -> This::Reading {
        self.read(304).await
    }

    /// Reads the value at 305.
    async fn http_read_305(&self) -> This::Reading {
        self.read(305).await
    }

    /// Reads the value at 306.
    async fn http_read_306(&self) -> This::Reading {
        self.read(306).await
    }

    /// Reads the value at 307.
    async fn http_read_307(&self) -> This::Reading {
        self.read(307).await
    }

    /// Reads the value at 308.
    async fn http_read_308(&self) -> This::Reading {
        self.read(308).await
    }

    /// Reads the value at 309.
    async fn http_read_309(&self) -> This::Reading {
        self.read(309).await
    }

    /// Reads the value at 310.
    async fn http_read_310(&self) -> This::Reading {
        self.read(310).await
    }

    /// Reads the value at 311.
    async fn http_read_311(&self) -> This::Reading {
        self.read(311).await
    }

    /// Reads the value at 312.
    async fn http_read_312(&self) -> This::Reading {
        self.read(312).await
    }

    /// Reads the value at 313.
    async fn http_read_313(&self) -> This::Reading {
        self.read(313).await
    }

    /// Reads the value at 314.
    async fn http_read_314(&self) -> This::Reading {
        self.read(314).await
    }

    /// Reads the value at 315.
    async fn http_read_315(&self) -> This::Reading {
        self.read(315).await
    }

    /// Reads the value at 316.
    async fn http_read_316(&self) -> This::Reading {
        self.read(316).await
    }

    /// Reads the value at 317.
    async fn http_read_317(&self) -> This::Reading {
        self.read(317).await
    }

    /// Reads the value at 318.
    async fn http_read_318(&self) -> This::Reading {
        self.read(318).await
    }

    /// Reads the value at 319.
    async fn http_read_319(&self) -> This::Reading {
        self.read(319).await
    }

    /// Reads the value at 320.
    async fn http_read_320(&self) -> This::Reading {
        self.read(320).await
    }

    /// Reads the value at 321.
    async fn http_read_321(&self) -> This::Reading {
        self.read(321).await
    }

    /// Reads the value at 322.
    async fn http_read_322(&self) -> This::Reading {
        self.read(322).await
    }

    /// Reads the value at 323.
    async fn http_read_323(&self) -> This::Reading {
        self.read(323).await
    }

    /// Reads the value at 324.
    async fn http_read_324(&self) -> This::Reading {
        self.read(324).await
    }

    /// Reads the value at 325.
    async fn http_read_325(&self) -> This::Reading {
        self.read(325).await
    }

    /// Reads the value at 326.
    async fn http_read_326(&self) -> This::Reading {
        self.read(326).await
    }

    /// Reads the value at 327.
    async fn http_read_327(&self) -> This::Reading {
        self.read(327).await
    }

    /// Reads the value at 328.
    async fn http_read_328(&self) -> This::Reading {
        self.read(328).await
    }

    /// Reads the value at 329.
    async fn http_read_329(&self) -> This::Reading {
        self.read(329).await
    }

    /// Reads the value at 330.
    async fn http_read_330(&self) -> This::Reading {
        self.read(330).await
    }

    /// Reads the value at 331.
    async fn http_read_331(&self) -> This::Reading {
        self.read(331).await
    }

    /// Reads the value at 332.
    async fn http_read_332(&self) -> This::Reading {
        self.read(332).await
    }

    /// Reads the value at 333.
    async fn http_read_333(&self) -> This::Reading {
        self.read(333).await
    }

    /// Reads the value at 334.
    async fn http_read_334(&self) -> This::Reading {
        self.read(334).await
    }

    /// Reads the value at 335.
    async fn http_read_335(&self) -> This::Reading {
        self.read(335).await
    }

    /// Reads the value at 336.
    async fn http_read_336(&self) -> This::Reading {
        self.read(336).await
    }

    /// Reads the value at 337.
    async fn http_read_337(&self) -> This::Reading {
        self.read(337).await
    }

    /// Reads the value at 338.
    async fn http_read_338(&self) -> This::Reading {
        self.read(338).await
    }

    /// Reads the value at 339.
    async fn http_read_339(&self) -> This::Reading {
        self.read(339).await
    }

    /// Reads the value at 340.
    async fn http_read_340(&self) -> This::Reading {
        self.read(340).await
    }

    /// Reads the value at 341.
    async fn http_read_341(&self) -> This::Reading {
        self.read(341).await
    }

    /// Reads the value at 342.
    async fn http_read_342(&self) -> This::Reading {
        self.read(342).await
    }

    /// Reads the value at 343.
    async fn http_read_343(&self) -> This::Reading {
        self.read(343).await
    }

    /// Reads the value at 344.
    async fn http_read_344(&self) -> This::Reading {
        self.read(344).await
    }

    /// Reads the value at 345.
    async fn http_read_345(&self) -> This::Reading {
        self.read(345).await
    }

    /// Reads the value at 346.
    async fn http_read_346(&self) -> This::Reading {
        self.read(346).await
    }

    /// Reads the value at 347.
    async fn http_read_347(&self) -> This::Reading {
        self.read(347).await
    }

    /// Reads the value at 348.
    async fn http_read_348(&self) -> This::Reading {
        self.read(348).await
    }

    /// Reads the value at 349.
    async fn http_read_349(&self) -> This::Reading {
        self.read(349).await
    }

    /// Reads the value at 350.
    async fn http_read_350(&self) -> This::Reading {
        self.read(350).await
    }

    /// Reads the value at 351.
    async fn http_read_351(&self) -> This::Reading {
        self.read(351).await
    }

    /// Reads the value at 352.
    async fn http_read_352(&self) -> This::Reading {
        self.read(352).await
    }

    /// Reads the value at 353.
    async fn http_read_353(&self) -> This::Reading {
        self.read(353).await
    }

    /// Reads the value at 354.
    async fn http_read_354(&self) -> This::Reading {
        self.read(354).await
    }

    /// Reads the value at 355.
    async fn http_read_355(&self) -> This::Reading {
        self.read(355).await
    }

    /// Reads the value at 356.
    async fn http_read_356(&self) -> This::Reading {
        self.read(356).await
    }

    /// Reads the value at 357.
    async fn http_read_357(&self) -> This::Reading {
        self.read(357).await
    }

    /// Reads the value at 358.
    async fn http_read_358(&self) -> This::Reading {
        self.read(358).await
    }

    /// Reads the value at 359.
    async fn http_read_359(&self) -> This::Reading {
        self.read(359).await
    }

    /// Reads the value at 360.
    async fn http_read_360(&self) -> This::Reading {
        self.read(360).await
    }

    /// Reads the value at 361.
    async fn http_read_361(&self) -> This::Reading {
        self.read(361).await
    }

    /// Reads the value at 362.
    async fn http_read_362(&self) -> This::Reading {
        self.read(362).await
    }

    /// Reads the value at 363.
    async fn http_read_363(&self) -> This::Reading {
        self.read(363).await
    }

    /// Reads the value at 364.
    async fn http_read_364(&self) -> This::Reading {
        self.read(364).await
    }

    /// Reads the value at 365.
    async fn http_read_365(&self) -> This::Reading {
        self.read(365).await
    }

    /// Reads the value at 366.
    async fn http_read_366(&self) -> This::Reading {
        self.read(366).await
    }

    /// Reads the value at 367.
    async fn http_read_367(&self) -> This::Reading {
        self.read(367).await
    }

    /// Reads the value at 368.
    async fn http_read_368(&self) -> This::Reading {
        self.read(368).await
    }

    /// Reads the value at 369.
    async fn http_read_369(&self) -> This::Reading {
        self.read(369).await
    }

    /// Reads the value at 370.
    async fn http_read_370(&self) -> This::Reading {
        self.read(370).await
    }

    /// Reads the value at 371.
    async fn http_read_371(&self) -> This::Reading {
        self.read(371).await
    }

    /// Reads the value at 372.
    async fn http_read_372(&self) -> This::Reading {
        self.read(372).await
    }

    /// Reads the value at 373.
    async fn http_read_373(&self) -> This::Reading {
        self.read(373).await
    }

    /// Reads the value at 374.
    async fn http_read_374(&self) -> This::Reading {
        self.read(374).await
    }

    /// Reads the value at 375.
    async fn http_read_375(&self) -> This::Reading {
        self.read(375).await
    }

    /// Reads the value at 376.
    async fn http_read_376(&self) -> This::Reading {
        self.read(376).await
    }

    /// Reads the value at 377.
    async fn http_read_377(&self) -> This::Reading {
        self.read(377).await
    }

    /// Reads the value at 378.
    async fn http_read_378(&self) -> This::Reading {
        self.read(378).await
    }

    /// Reads the value at 379.
    async fn http_read_379(&self) -> This::Reading {
        self.read(379).await
    }

    /// Reads the value at 380.
    async fn http_read_380(&self) -> This::Reading {
        self.read(380).await
    }

    /// Reads the value at 381.
    async fn http_read_381(&self) -> This::Reading {
        self.read(381).await
    }

    /// Reads the value at 382.
    async fn http_read_382(&self) -> This::Reading {
        self.read(382).await
    }

    /// Reads the value at 383.
    async fn http_read_383(&self) -> This::Reading {
        self.read(383).await
    }

    /// Reads the value at 384.
    async fn http_read_384(&self) -> This::Reading {
        self.read(384).await
    }

    /// Reads the value at 385.
    async fn http_read_385(&self) -> This::Reading {
        self.read(385).await
    }

    /// Reads the value at 386.
    async fn http_read_386(&self) -> This::Reading {
        self.read(386).await
    }

    /// Reads the value at 387.
    async fn http_read_387(&self) -> This::Reading {
        self.read(387).await
    }

    /// Reads the value at 388.
    async fn http_read_388(&self) -> This::Reading {
        self.read(388).await
    }

    /// Reads the value at 389.
    async fn http_read_389(&self) -> This::Reading {
        self.read(389).await
    }

    /// Reads the value at 390.
    async fn http_read_390(&self) -> This::Reading {
        self.read(390).await
    }

    /// Reads the value at 391.
    async fn http_read_391(&self) -> This::Reading {
        self.read(391).await
    }

    /// Reads the value at 392.
    async fn http_read_392(&self) -> This::Reading {
        self.read(392).await
    }

    /// Reads the value at 393.
    async fn http_read_393(&self) -> This::Reading {
        self.read(393).await
    }

    /// Reads the value at 394.
    async fn http_read_394(&self) -> This::Reading {
        self.read(394).await
    }

    /// Reads the value at 395.
    async fn http_read_395(&self) -> This::Reading {
        self.read(395).await
    }

    /// Reads the value at 396.
    async fn http_read_396(&self) -> This::Reading {
        self.read(396).await
    }

    /// Reads the value at 397.
    async fn http_read_397(&self) -> This::Reading {
        self.read(397).await
    }

    /// Reads the value at 398.
    async fn http_read_398(&self) -> This::Reading {
        self.read(398).await
    }

    /// Reads the value at 399.
    async fn http_read_399(&self) -> This::Reading {
        self.read(399).await
    }

    /// Reads the value at 400.
    async fn http_read_400(&self) -> This::Reading {
        self.read(400).await
    }

    /// Reads the value at 401.
    async fn http_read_401(&self) -> This::Reading {
        self.read(401).await
    }

    /// Reads the value at 402.
    async fn http_read_402(&self) -> This::Reading {
        self.read(402).await
    }

    /// Reads the value at 403.
    async fn http_read_403(&self) -> This::Reading {
        self.read(403).await
    }

    /// Reads the value at 404.
    async fn http_read_404(&self) -> This::Reading {
        self.read(404).await
    }

    /// Reads the value at 405.
    async fn http_read_405(&self) -> This::Reading {
        self.read(405).await
    }

    /// Reads the value at 406.
    async fn http_read_406(&self) -> This::Reading {
        self.read(406).await
    }

    /// Reads the value at 407.
    async fn http_read_407(&self) -> This::Reading {
        self.read(407).await
    }

    /// Reads the value at 408.
    async fn http_read_408(&self) -> This::Reading {
        self.read(408).await
    }

    /// Reads the value at 409.
    async fn http_read_409(&self) -> This::Reading {
        self.read(409).await
    }

    /// Reads the value at 410.
    async fn http_read_410(&self) -> This::Reading {
        self.read(410).await
    }

    /// Reads the value at 411.
    async fn http_read_411(&self) -> This::Reading {
        self.read(411).await
    }

    /// Reads the value at 412.
    async fn http_read_412(&self) -> This::Reading {
        self.read(412).await
    }

    /// Reads the value at 413.
    async fn http_read_413(&self) -> This::Reading {
        self.read(413).await
    }

    /// Reads the value at 414.
    async fn http_read_414(&self) -> This::Reading {
        self.read(414).await
    }

    /// Reads the value at 415.
    async fn http_read_415(&self) -> This::Reading {
        self.read(415).await
    }

    /// Reads the value at 416.
    async fn http_read_416(&self) -> This::Reading {
        self.read(416).await
    }

    /// Reads the value at 417.
    async fn http_read_417(&self) -> This::Reading {
        self.read(417).await
    }

    /// Reads the value at 418.
    async fn http_read_418(&self) -> This::Reading {
        self.read(418).await
    }

    /// Reads the value at 419.
    async fn http_read_419(&self) -> This::Reading {
        self.read(419).await
    }

    /// Reads the value at 420.
    async fn http_read_420(&self) -> This::Reading {
        self.read(420).await
    }

    /// Reads the value at 421.
    async fn http_read_421(&self) -> This::Reading {
        self.read(421).await
    }

    /// Reads the value at 422.
    async fn http_read_422(&self) -> This::Reading {
        self.read(422).await
    }

    /// Reads the value at 423.
    async fn http_read_423(&self) -> This::Reading {
        self.read(423).await
    }

    /// Reads the value at 424.
    async fn http_read_424(&self) -> This::Reading {
        self.read(424).await
    }

    /// Reads the value at 425.
    async fn http_read_425(&self) -> This::Reading {
        self.read(425).await
    }

    /// Reads the value at 426.
    async fn http_read_426(&self) -> This::Reading {
        self.read(426).await
    }

    /// Reads the value at 427.
    async fn http_read_427(&self) -> This::Reading {
        self.read(427).await
    }

    /// Reads the value at 428.
    async fn http_read_428(&self) -> This::Reading {
        self.read(428).await
    }

    /// Reads the value at 429.
    async fn http_read_429(&self) -> This::Reading {
        self.read(429).await
    }

    /// Reads the value at 430.
    async fn http_read_430(&self) -> This::Reading {
        self.read(430).await
    }

    /// Reads the value at 431.
    async fn http_read_431(&self) -> This::Reading {
        self.read(431).await
    }

    /// Reads the value at 432.
    async fn http_read_432(&self) -> This::Reading {
        self.read(432).await
    }

    /// Reads the value at 433.
    async fn http_read_433(&self) -> This::Reading {
        self.read(433).await
    }

    /// Reads the value at 434.
    async fn http_read_434(&self) -> This::Reading {
        self.read(434).await
    }

    /// Reads the value at 435.
    async fn http_read_435(&self) -> This::Reading {
        self.read(435).await
    }

    /// Reads the value at 436.
    async fn http_read_436(&self) -> This::Reading {
        self.read(436).await
    }

    /// Reads the value at 437.
    async fn http_read_437(&self) -> This::Reading {
        self.read(437).await
    }

    /// Reads the value at 438.
    async fn http_read_438(&self) -> This::Reading {
        self.read(438).await
    }

    /// Reads the value at 439.
    async fn http_read_439(&self) -> This::Reading {
        self.read(439).await
    }

    /// Reads the value at 440.
    async fn http_read_440(&self) -> This::Reading {
        self.read(440).await
    }

    /// Reads the value at 441.
    async fn http_read_441(&self) -> This::Reading {
        self.read(441).await
    }

    /// Reads the value at 442.
    async fn http_read_442(&self) -> This::Reading {
        self.read(442).await
    }

    /// Reads the value at 443.
    async fn http_read_443(&self) -> This::Reading {
        self.read(443).await
    }

    /// Reads the value at 444.
    async fn http_read_444(&self) -> This::Reading {
        self.read(444).await
    }

    /// Reads the value at 445.
    async fn http_read_445(&self) -> This::Reading {
        self.read(445).await
    }

    /// Reads the value at 446.
    async fn http_read_446(&self) -> This::Reading {
        self.read(446).await
    }

    /// Reads the value at 447.
    async fn http_read_447(&self) -> This::Reading {
        self.read(447).await
    }

    /// Reads the value at 448.
    async fn http_read_448(&self) -> This::Reading {
        self.read(448).await
    }

    /// Reads the value at 449.
    async fn http_read_449(&self) -> This::Reading {
        self.read(449).await
    }

    /// Reads the value at 450.
    async fn http_read_450(&self) -> This::Reading {
        self.read(450).await
    }

    /// Reads the value at 451.
    async fn http_read_451(&self) -> This::Reading {
        self.read(451).await
    }

    /// Reads the value at 452.
    async fn http_read_452(&self) -> This::Reading {
        self.read(452).await
    }

    /// Reads the value at 453.
    async fn http_read_453(&self) -> This::Reading {
        self.read(453).await
    }

    /// Reads the value at 454.
    async fn http_read_454(&self) -> This::Reading {
        self.read(454).await
    }

    /// Reads the value at 455.
    async fn http_read_455(&self) -> This::Reading {
        self.read(455).await
    }

    /// Reads the value at 456.
    async fn http_read_456(&self) -> This::Reading {
        self.read(456).await
    }

    /// Reads the value at 457.
    async fn http_read_457(&self) -> This::Reading {
        self.read(457).await
    }

    /// Reads the value at 458.
    async fn http_read_458(&self) -> This::Reading {
        self.read(458).await
    }

    /// Reads the value at 459.
    async fn http_read_459(&self) -> This::Reading {
        self.read(459).await
    }

    /// Reads the value at 460.
    async fn http_read_460(&self) -> This::Reading {
        self.read(460).await
    }

    /// Reads the value at 461.
    async fn http_read_461(&self) -> This::Reading {
        self.read(461).await
    }

    /// Reads the value at 462.
    async fn http_read_462(&self) -> This::Reading {
        self.read(462).await
    }

    /// Reads the value at 463.
    async fn http_read_463(&self) -> This::Reading {
        self.read(463).await
    }

    /// Reads the value at 464.
    async fn http_read_464(&self) -> This::Reading {
        self.read(464).await
    }

    /// Reads the value at 465.
    async fn http_read_465(&self) -> This::Reading {
        self.read(465).await
    }

    /// Reads the value at 466.
    async fn http_read_466(&self) -> This::Reading {
        self.read(466).await
    }

    /// Reads the value at 467.
    async fn http_read_467(&self) -> This::Reading {
        self.read(467).await
    }

    /// Reads the value at 468.
    async fn http_read_468(&self) -> This::Reading {
        self.read(468).await
    }

    /// Reads the value at 469.
    async fn http_read_469(&self) -> This::Reading {
        self.read(469).await
    }

    /// Reads the value at 470.
    async fn http_read_470(&self) -> This::Reading {
        self.read(470).await
    }

    /// Reads the value at 471.
    async fn http_read_471(&self) -> This::Reading {
        self.read(471).await
    }

    /// Reads the value at 472.
    async fn http_read_472(&self) -> This::Reading {
        self.read(472).await
    }

    /// Reads the value at 473.
    async fn http_read_473(&self) -> This::Reading {
        self.read(473).await
    }

    /// Reads the value at 474.
    async fn http_read_474(&self) -> This::Reading {
        self.read(474).await
    }

    /// Reads the value at 475.
    async fn http_read_475(&self) -> This::Reading {
        self.read(475).await
    }

    /// Reads the value at 476.
    async fn http_read_476(&self) -> This::Reading {
        self.read(476).await
    }

    /// Reads the value at 477.
    async fn http_read_477(&self) -> This::Reading {
        self.read(477).await
    }

    /// Reads the value at 478.
    async fn http_read_478(&self) -> This::Reading {
        self.read(478).await
    }

    /// Reads the value at 479.
    async fn http_read_479(&self) -> This::Reading {
        self.read(479).await
    }

    /// Reads the value at 480.
    async fn http_read_480(&self) -> This::Reading {
        self.read(480).await
    }

    /// Reads the value at 481.
    async fn http_read_481(&self) -> This::Reading {
        self.read(481).await
    }

    /// Reads the value at 482.
    async fn http_read_482(&self) -> This::Reading {
        self.read(482).await
    }

    /// Reads the value at 483.
    async fn http_read_483(&self) -> This::Reading {
        self.read(483).await
    }

    /// Reads the value at 484.
    async fn http_read_484(&self) -> This::Reading {
        self.read(484).await
    }

    /// Reads the value at 485.
    async fn http_read_485(&self) -> This::Reading {
        self.read(485).await
    }

    /// Reads the value at 486.
    async fn http_read_486(&self) -> This::Reading {
        self.read(486).await
    }

    /// Reads the value at 487.
    async fn http_read_487(&self) -> This::Reading {
        self.read(487).await
    }

    /// Reads the value at 488.
    async fn http_read_488(&self) -> This::Reading {
        self.read(488).await
    }

    /// Reads the value at 489.
    async fn http_read_489(&self) -> This::Reading {
        self.read(489).await
    }

    /// Reads the value at 490.
    async fn http_read_490(&self) -> This::Reading {
        self.read(490).await
    }

    /// Reads the value at 491.
    async fn http_read_491(&self) -> This::Reading {
        self.read(491).await
    }

    /// Reads the value at 492.
    async fn http_read_492(&self) -> This::Reading {
        self.read(492).await
    }

    /// Reads the value at 493.
    async fn http_read_493(&self) -> This::Reading {
        self.read(493).await
    }

    /// Reads the value at 494.
    async fn http_read_494(&self) -> This::Reading {
        self.read(494).await
    }

    /// Reads the value at 495.
    async fn http_read_495(&self) -> This::Reading {
        self.read(495).await
    }

    /// Reads the value at 496.
    async fn http_read_496(&self) -> This::Reading {
        self.read(496).await
    }

    /// Reads the value at 497.
    async fn http_read_497(&self) -> This::Reading {
        self.read(497).await
    }

    /// Reads the value at 498.
    async fn http_read_498(&self) -> This::Reading {
        self.read(498).await
    }

    /// Reads the value at 499.
    async fn http_read_499(&self) -> This::Reading {
        self.read(499).await
    }
}

/// Declares every route in one chain, which is the shape being guarded.
#[ext(name = ManyApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    /// Declares all 500 routes.
    fn many_api<Alg>(&self)
    where
        Alg: ReadingAlg,
    {
        self.routes()
            .get("/read/0", self.op(Alg::http_read_0).json())
            .get("/read/1", self.op(Alg::http_read_1).json())
            .get("/read/2", self.op(Alg::http_read_2).json())
            .get("/read/3", self.op(Alg::http_read_3).json())
            .get("/read/4", self.op(Alg::http_read_4).json())
            .get("/read/5", self.op(Alg::http_read_5).json())
            .get("/read/6", self.op(Alg::http_read_6).json())
            .get("/read/7", self.op(Alg::http_read_7).json())
            .get("/read/8", self.op(Alg::http_read_8).json())
            .get("/read/9", self.op(Alg::http_read_9).json())
            .get("/read/10", self.op(Alg::http_read_10).json())
            .get("/read/11", self.op(Alg::http_read_11).json())
            .get("/read/12", self.op(Alg::http_read_12).json())
            .get("/read/13", self.op(Alg::http_read_13).json())
            .get("/read/14", self.op(Alg::http_read_14).json())
            .get("/read/15", self.op(Alg::http_read_15).json())
            .get("/read/16", self.op(Alg::http_read_16).json())
            .get("/read/17", self.op(Alg::http_read_17).json())
            .get("/read/18", self.op(Alg::http_read_18).json())
            .get("/read/19", self.op(Alg::http_read_19).json())
            .get("/read/20", self.op(Alg::http_read_20).json())
            .get("/read/21", self.op(Alg::http_read_21).json())
            .get("/read/22", self.op(Alg::http_read_22).json())
            .get("/read/23", self.op(Alg::http_read_23).json())
            .get("/read/24", self.op(Alg::http_read_24).json())
            .get("/read/25", self.op(Alg::http_read_25).json())
            .get("/read/26", self.op(Alg::http_read_26).json())
            .get("/read/27", self.op(Alg::http_read_27).json())
            .get("/read/28", self.op(Alg::http_read_28).json())
            .get("/read/29", self.op(Alg::http_read_29).json())
            .get("/read/30", self.op(Alg::http_read_30).json())
            .get("/read/31", self.op(Alg::http_read_31).json())
            .get("/read/32", self.op(Alg::http_read_32).json())
            .get("/read/33", self.op(Alg::http_read_33).json())
            .get("/read/34", self.op(Alg::http_read_34).json())
            .get("/read/35", self.op(Alg::http_read_35).json())
            .get("/read/36", self.op(Alg::http_read_36).json())
            .get("/read/37", self.op(Alg::http_read_37).json())
            .get("/read/38", self.op(Alg::http_read_38).json())
            .get("/read/39", self.op(Alg::http_read_39).json())
            .get("/read/40", self.op(Alg::http_read_40).json())
            .get("/read/41", self.op(Alg::http_read_41).json())
            .get("/read/42", self.op(Alg::http_read_42).json())
            .get("/read/43", self.op(Alg::http_read_43).json())
            .get("/read/44", self.op(Alg::http_read_44).json())
            .get("/read/45", self.op(Alg::http_read_45).json())
            .get("/read/46", self.op(Alg::http_read_46).json())
            .get("/read/47", self.op(Alg::http_read_47).json())
            .get("/read/48", self.op(Alg::http_read_48).json())
            .get("/read/49", self.op(Alg::http_read_49).json())
            .get("/read/50", self.op(Alg::http_read_50).json())
            .get("/read/51", self.op(Alg::http_read_51).json())
            .get("/read/52", self.op(Alg::http_read_52).json())
            .get("/read/53", self.op(Alg::http_read_53).json())
            .get("/read/54", self.op(Alg::http_read_54).json())
            .get("/read/55", self.op(Alg::http_read_55).json())
            .get("/read/56", self.op(Alg::http_read_56).json())
            .get("/read/57", self.op(Alg::http_read_57).json())
            .get("/read/58", self.op(Alg::http_read_58).json())
            .get("/read/59", self.op(Alg::http_read_59).json())
            .get("/read/60", self.op(Alg::http_read_60).json())
            .get("/read/61", self.op(Alg::http_read_61).json())
            .get("/read/62", self.op(Alg::http_read_62).json())
            .get("/read/63", self.op(Alg::http_read_63).json())
            .get("/read/64", self.op(Alg::http_read_64).json())
            .get("/read/65", self.op(Alg::http_read_65).json())
            .get("/read/66", self.op(Alg::http_read_66).json())
            .get("/read/67", self.op(Alg::http_read_67).json())
            .get("/read/68", self.op(Alg::http_read_68).json())
            .get("/read/69", self.op(Alg::http_read_69).json())
            .get("/read/70", self.op(Alg::http_read_70).json())
            .get("/read/71", self.op(Alg::http_read_71).json())
            .get("/read/72", self.op(Alg::http_read_72).json())
            .get("/read/73", self.op(Alg::http_read_73).json())
            .get("/read/74", self.op(Alg::http_read_74).json())
            .get("/read/75", self.op(Alg::http_read_75).json())
            .get("/read/76", self.op(Alg::http_read_76).json())
            .get("/read/77", self.op(Alg::http_read_77).json())
            .get("/read/78", self.op(Alg::http_read_78).json())
            .get("/read/79", self.op(Alg::http_read_79).json())
            .get("/read/80", self.op(Alg::http_read_80).json())
            .get("/read/81", self.op(Alg::http_read_81).json())
            .get("/read/82", self.op(Alg::http_read_82).json())
            .get("/read/83", self.op(Alg::http_read_83).json())
            .get("/read/84", self.op(Alg::http_read_84).json())
            .get("/read/85", self.op(Alg::http_read_85).json())
            .get("/read/86", self.op(Alg::http_read_86).json())
            .get("/read/87", self.op(Alg::http_read_87).json())
            .get("/read/88", self.op(Alg::http_read_88).json())
            .get("/read/89", self.op(Alg::http_read_89).json())
            .get("/read/90", self.op(Alg::http_read_90).json())
            .get("/read/91", self.op(Alg::http_read_91).json())
            .get("/read/92", self.op(Alg::http_read_92).json())
            .get("/read/93", self.op(Alg::http_read_93).json())
            .get("/read/94", self.op(Alg::http_read_94).json())
            .get("/read/95", self.op(Alg::http_read_95).json())
            .get("/read/96", self.op(Alg::http_read_96).json())
            .get("/read/97", self.op(Alg::http_read_97).json())
            .get("/read/98", self.op(Alg::http_read_98).json())
            .get("/read/99", self.op(Alg::http_read_99).json())
            .get("/read/100", self.op(Alg::http_read_100).json())
            .get("/read/101", self.op(Alg::http_read_101).json())
            .get("/read/102", self.op(Alg::http_read_102).json())
            .get("/read/103", self.op(Alg::http_read_103).json())
            .get("/read/104", self.op(Alg::http_read_104).json())
            .get("/read/105", self.op(Alg::http_read_105).json())
            .get("/read/106", self.op(Alg::http_read_106).json())
            .get("/read/107", self.op(Alg::http_read_107).json())
            .get("/read/108", self.op(Alg::http_read_108).json())
            .get("/read/109", self.op(Alg::http_read_109).json())
            .get("/read/110", self.op(Alg::http_read_110).json())
            .get("/read/111", self.op(Alg::http_read_111).json())
            .get("/read/112", self.op(Alg::http_read_112).json())
            .get("/read/113", self.op(Alg::http_read_113).json())
            .get("/read/114", self.op(Alg::http_read_114).json())
            .get("/read/115", self.op(Alg::http_read_115).json())
            .get("/read/116", self.op(Alg::http_read_116).json())
            .get("/read/117", self.op(Alg::http_read_117).json())
            .get("/read/118", self.op(Alg::http_read_118).json())
            .get("/read/119", self.op(Alg::http_read_119).json())
            .get("/read/120", self.op(Alg::http_read_120).json())
            .get("/read/121", self.op(Alg::http_read_121).json())
            .get("/read/122", self.op(Alg::http_read_122).json())
            .get("/read/123", self.op(Alg::http_read_123).json())
            .get("/read/124", self.op(Alg::http_read_124).json())
            .get("/read/125", self.op(Alg::http_read_125).json())
            .get("/read/126", self.op(Alg::http_read_126).json())
            .get("/read/127", self.op(Alg::http_read_127).json())
            .get("/read/128", self.op(Alg::http_read_128).json())
            .get("/read/129", self.op(Alg::http_read_129).json())
            .get("/read/130", self.op(Alg::http_read_130).json())
            .get("/read/131", self.op(Alg::http_read_131).json())
            .get("/read/132", self.op(Alg::http_read_132).json())
            .get("/read/133", self.op(Alg::http_read_133).json())
            .get("/read/134", self.op(Alg::http_read_134).json())
            .get("/read/135", self.op(Alg::http_read_135).json())
            .get("/read/136", self.op(Alg::http_read_136).json())
            .get("/read/137", self.op(Alg::http_read_137).json())
            .get("/read/138", self.op(Alg::http_read_138).json())
            .get("/read/139", self.op(Alg::http_read_139).json())
            .get("/read/140", self.op(Alg::http_read_140).json())
            .get("/read/141", self.op(Alg::http_read_141).json())
            .get("/read/142", self.op(Alg::http_read_142).json())
            .get("/read/143", self.op(Alg::http_read_143).json())
            .get("/read/144", self.op(Alg::http_read_144).json())
            .get("/read/145", self.op(Alg::http_read_145).json())
            .get("/read/146", self.op(Alg::http_read_146).json())
            .get("/read/147", self.op(Alg::http_read_147).json())
            .get("/read/148", self.op(Alg::http_read_148).json())
            .get("/read/149", self.op(Alg::http_read_149).json())
            .get("/read/150", self.op(Alg::http_read_150).json())
            .get("/read/151", self.op(Alg::http_read_151).json())
            .get("/read/152", self.op(Alg::http_read_152).json())
            .get("/read/153", self.op(Alg::http_read_153).json())
            .get("/read/154", self.op(Alg::http_read_154).json())
            .get("/read/155", self.op(Alg::http_read_155).json())
            .get("/read/156", self.op(Alg::http_read_156).json())
            .get("/read/157", self.op(Alg::http_read_157).json())
            .get("/read/158", self.op(Alg::http_read_158).json())
            .get("/read/159", self.op(Alg::http_read_159).json())
            .get("/read/160", self.op(Alg::http_read_160).json())
            .get("/read/161", self.op(Alg::http_read_161).json())
            .get("/read/162", self.op(Alg::http_read_162).json())
            .get("/read/163", self.op(Alg::http_read_163).json())
            .get("/read/164", self.op(Alg::http_read_164).json())
            .get("/read/165", self.op(Alg::http_read_165).json())
            .get("/read/166", self.op(Alg::http_read_166).json())
            .get("/read/167", self.op(Alg::http_read_167).json())
            .get("/read/168", self.op(Alg::http_read_168).json())
            .get("/read/169", self.op(Alg::http_read_169).json())
            .get("/read/170", self.op(Alg::http_read_170).json())
            .get("/read/171", self.op(Alg::http_read_171).json())
            .get("/read/172", self.op(Alg::http_read_172).json())
            .get("/read/173", self.op(Alg::http_read_173).json())
            .get("/read/174", self.op(Alg::http_read_174).json())
            .get("/read/175", self.op(Alg::http_read_175).json())
            .get("/read/176", self.op(Alg::http_read_176).json())
            .get("/read/177", self.op(Alg::http_read_177).json())
            .get("/read/178", self.op(Alg::http_read_178).json())
            .get("/read/179", self.op(Alg::http_read_179).json())
            .get("/read/180", self.op(Alg::http_read_180).json())
            .get("/read/181", self.op(Alg::http_read_181).json())
            .get("/read/182", self.op(Alg::http_read_182).json())
            .get("/read/183", self.op(Alg::http_read_183).json())
            .get("/read/184", self.op(Alg::http_read_184).json())
            .get("/read/185", self.op(Alg::http_read_185).json())
            .get("/read/186", self.op(Alg::http_read_186).json())
            .get("/read/187", self.op(Alg::http_read_187).json())
            .get("/read/188", self.op(Alg::http_read_188).json())
            .get("/read/189", self.op(Alg::http_read_189).json())
            .get("/read/190", self.op(Alg::http_read_190).json())
            .get("/read/191", self.op(Alg::http_read_191).json())
            .get("/read/192", self.op(Alg::http_read_192).json())
            .get("/read/193", self.op(Alg::http_read_193).json())
            .get("/read/194", self.op(Alg::http_read_194).json())
            .get("/read/195", self.op(Alg::http_read_195).json())
            .get("/read/196", self.op(Alg::http_read_196).json())
            .get("/read/197", self.op(Alg::http_read_197).json())
            .get("/read/198", self.op(Alg::http_read_198).json())
            .get("/read/199", self.op(Alg::http_read_199).json())
            .get("/read/200", self.op(Alg::http_read_200).json())
            .get("/read/201", self.op(Alg::http_read_201).json())
            .get("/read/202", self.op(Alg::http_read_202).json())
            .get("/read/203", self.op(Alg::http_read_203).json())
            .get("/read/204", self.op(Alg::http_read_204).json())
            .get("/read/205", self.op(Alg::http_read_205).json())
            .get("/read/206", self.op(Alg::http_read_206).json())
            .get("/read/207", self.op(Alg::http_read_207).json())
            .get("/read/208", self.op(Alg::http_read_208).json())
            .get("/read/209", self.op(Alg::http_read_209).json())
            .get("/read/210", self.op(Alg::http_read_210).json())
            .get("/read/211", self.op(Alg::http_read_211).json())
            .get("/read/212", self.op(Alg::http_read_212).json())
            .get("/read/213", self.op(Alg::http_read_213).json())
            .get("/read/214", self.op(Alg::http_read_214).json())
            .get("/read/215", self.op(Alg::http_read_215).json())
            .get("/read/216", self.op(Alg::http_read_216).json())
            .get("/read/217", self.op(Alg::http_read_217).json())
            .get("/read/218", self.op(Alg::http_read_218).json())
            .get("/read/219", self.op(Alg::http_read_219).json())
            .get("/read/220", self.op(Alg::http_read_220).json())
            .get("/read/221", self.op(Alg::http_read_221).json())
            .get("/read/222", self.op(Alg::http_read_222).json())
            .get("/read/223", self.op(Alg::http_read_223).json())
            .get("/read/224", self.op(Alg::http_read_224).json())
            .get("/read/225", self.op(Alg::http_read_225).json())
            .get("/read/226", self.op(Alg::http_read_226).json())
            .get("/read/227", self.op(Alg::http_read_227).json())
            .get("/read/228", self.op(Alg::http_read_228).json())
            .get("/read/229", self.op(Alg::http_read_229).json())
            .get("/read/230", self.op(Alg::http_read_230).json())
            .get("/read/231", self.op(Alg::http_read_231).json())
            .get("/read/232", self.op(Alg::http_read_232).json())
            .get("/read/233", self.op(Alg::http_read_233).json())
            .get("/read/234", self.op(Alg::http_read_234).json())
            .get("/read/235", self.op(Alg::http_read_235).json())
            .get("/read/236", self.op(Alg::http_read_236).json())
            .get("/read/237", self.op(Alg::http_read_237).json())
            .get("/read/238", self.op(Alg::http_read_238).json())
            .get("/read/239", self.op(Alg::http_read_239).json())
            .get("/read/240", self.op(Alg::http_read_240).json())
            .get("/read/241", self.op(Alg::http_read_241).json())
            .get("/read/242", self.op(Alg::http_read_242).json())
            .get("/read/243", self.op(Alg::http_read_243).json())
            .get("/read/244", self.op(Alg::http_read_244).json())
            .get("/read/245", self.op(Alg::http_read_245).json())
            .get("/read/246", self.op(Alg::http_read_246).json())
            .get("/read/247", self.op(Alg::http_read_247).json())
            .get("/read/248", self.op(Alg::http_read_248).json())
            .get("/read/249", self.op(Alg::http_read_249).json())
            .get("/read/250", self.op(Alg::http_read_250).json())
            .get("/read/251", self.op(Alg::http_read_251).json())
            .get("/read/252", self.op(Alg::http_read_252).json())
            .get("/read/253", self.op(Alg::http_read_253).json())
            .get("/read/254", self.op(Alg::http_read_254).json())
            .get("/read/255", self.op(Alg::http_read_255).json())
            .get("/read/256", self.op(Alg::http_read_256).json())
            .get("/read/257", self.op(Alg::http_read_257).json())
            .get("/read/258", self.op(Alg::http_read_258).json())
            .get("/read/259", self.op(Alg::http_read_259).json())
            .get("/read/260", self.op(Alg::http_read_260).json())
            .get("/read/261", self.op(Alg::http_read_261).json())
            .get("/read/262", self.op(Alg::http_read_262).json())
            .get("/read/263", self.op(Alg::http_read_263).json())
            .get("/read/264", self.op(Alg::http_read_264).json())
            .get("/read/265", self.op(Alg::http_read_265).json())
            .get("/read/266", self.op(Alg::http_read_266).json())
            .get("/read/267", self.op(Alg::http_read_267).json())
            .get("/read/268", self.op(Alg::http_read_268).json())
            .get("/read/269", self.op(Alg::http_read_269).json())
            .get("/read/270", self.op(Alg::http_read_270).json())
            .get("/read/271", self.op(Alg::http_read_271).json())
            .get("/read/272", self.op(Alg::http_read_272).json())
            .get("/read/273", self.op(Alg::http_read_273).json())
            .get("/read/274", self.op(Alg::http_read_274).json())
            .get("/read/275", self.op(Alg::http_read_275).json())
            .get("/read/276", self.op(Alg::http_read_276).json())
            .get("/read/277", self.op(Alg::http_read_277).json())
            .get("/read/278", self.op(Alg::http_read_278).json())
            .get("/read/279", self.op(Alg::http_read_279).json())
            .get("/read/280", self.op(Alg::http_read_280).json())
            .get("/read/281", self.op(Alg::http_read_281).json())
            .get("/read/282", self.op(Alg::http_read_282).json())
            .get("/read/283", self.op(Alg::http_read_283).json())
            .get("/read/284", self.op(Alg::http_read_284).json())
            .get("/read/285", self.op(Alg::http_read_285).json())
            .get("/read/286", self.op(Alg::http_read_286).json())
            .get("/read/287", self.op(Alg::http_read_287).json())
            .get("/read/288", self.op(Alg::http_read_288).json())
            .get("/read/289", self.op(Alg::http_read_289).json())
            .get("/read/290", self.op(Alg::http_read_290).json())
            .get("/read/291", self.op(Alg::http_read_291).json())
            .get("/read/292", self.op(Alg::http_read_292).json())
            .get("/read/293", self.op(Alg::http_read_293).json())
            .get("/read/294", self.op(Alg::http_read_294).json())
            .get("/read/295", self.op(Alg::http_read_295).json())
            .get("/read/296", self.op(Alg::http_read_296).json())
            .get("/read/297", self.op(Alg::http_read_297).json())
            .get("/read/298", self.op(Alg::http_read_298).json())
            .get("/read/299", self.op(Alg::http_read_299).json())
            .get("/read/300", self.op(Alg::http_read_300).json())
            .get("/read/301", self.op(Alg::http_read_301).json())
            .get("/read/302", self.op(Alg::http_read_302).json())
            .get("/read/303", self.op(Alg::http_read_303).json())
            .get("/read/304", self.op(Alg::http_read_304).json())
            .get("/read/305", self.op(Alg::http_read_305).json())
            .get("/read/306", self.op(Alg::http_read_306).json())
            .get("/read/307", self.op(Alg::http_read_307).json())
            .get("/read/308", self.op(Alg::http_read_308).json())
            .get("/read/309", self.op(Alg::http_read_309).json())
            .get("/read/310", self.op(Alg::http_read_310).json())
            .get("/read/311", self.op(Alg::http_read_311).json())
            .get("/read/312", self.op(Alg::http_read_312).json())
            .get("/read/313", self.op(Alg::http_read_313).json())
            .get("/read/314", self.op(Alg::http_read_314).json())
            .get("/read/315", self.op(Alg::http_read_315).json())
            .get("/read/316", self.op(Alg::http_read_316).json())
            .get("/read/317", self.op(Alg::http_read_317).json())
            .get("/read/318", self.op(Alg::http_read_318).json())
            .get("/read/319", self.op(Alg::http_read_319).json())
            .get("/read/320", self.op(Alg::http_read_320).json())
            .get("/read/321", self.op(Alg::http_read_321).json())
            .get("/read/322", self.op(Alg::http_read_322).json())
            .get("/read/323", self.op(Alg::http_read_323).json())
            .get("/read/324", self.op(Alg::http_read_324).json())
            .get("/read/325", self.op(Alg::http_read_325).json())
            .get("/read/326", self.op(Alg::http_read_326).json())
            .get("/read/327", self.op(Alg::http_read_327).json())
            .get("/read/328", self.op(Alg::http_read_328).json())
            .get("/read/329", self.op(Alg::http_read_329).json())
            .get("/read/330", self.op(Alg::http_read_330).json())
            .get("/read/331", self.op(Alg::http_read_331).json())
            .get("/read/332", self.op(Alg::http_read_332).json())
            .get("/read/333", self.op(Alg::http_read_333).json())
            .get("/read/334", self.op(Alg::http_read_334).json())
            .get("/read/335", self.op(Alg::http_read_335).json())
            .get("/read/336", self.op(Alg::http_read_336).json())
            .get("/read/337", self.op(Alg::http_read_337).json())
            .get("/read/338", self.op(Alg::http_read_338).json())
            .get("/read/339", self.op(Alg::http_read_339).json())
            .get("/read/340", self.op(Alg::http_read_340).json())
            .get("/read/341", self.op(Alg::http_read_341).json())
            .get("/read/342", self.op(Alg::http_read_342).json())
            .get("/read/343", self.op(Alg::http_read_343).json())
            .get("/read/344", self.op(Alg::http_read_344).json())
            .get("/read/345", self.op(Alg::http_read_345).json())
            .get("/read/346", self.op(Alg::http_read_346).json())
            .get("/read/347", self.op(Alg::http_read_347).json())
            .get("/read/348", self.op(Alg::http_read_348).json())
            .get("/read/349", self.op(Alg::http_read_349).json())
            .get("/read/350", self.op(Alg::http_read_350).json())
            .get("/read/351", self.op(Alg::http_read_351).json())
            .get("/read/352", self.op(Alg::http_read_352).json())
            .get("/read/353", self.op(Alg::http_read_353).json())
            .get("/read/354", self.op(Alg::http_read_354).json())
            .get("/read/355", self.op(Alg::http_read_355).json())
            .get("/read/356", self.op(Alg::http_read_356).json())
            .get("/read/357", self.op(Alg::http_read_357).json())
            .get("/read/358", self.op(Alg::http_read_358).json())
            .get("/read/359", self.op(Alg::http_read_359).json())
            .get("/read/360", self.op(Alg::http_read_360).json())
            .get("/read/361", self.op(Alg::http_read_361).json())
            .get("/read/362", self.op(Alg::http_read_362).json())
            .get("/read/363", self.op(Alg::http_read_363).json())
            .get("/read/364", self.op(Alg::http_read_364).json())
            .get("/read/365", self.op(Alg::http_read_365).json())
            .get("/read/366", self.op(Alg::http_read_366).json())
            .get("/read/367", self.op(Alg::http_read_367).json())
            .get("/read/368", self.op(Alg::http_read_368).json())
            .get("/read/369", self.op(Alg::http_read_369).json())
            .get("/read/370", self.op(Alg::http_read_370).json())
            .get("/read/371", self.op(Alg::http_read_371).json())
            .get("/read/372", self.op(Alg::http_read_372).json())
            .get("/read/373", self.op(Alg::http_read_373).json())
            .get("/read/374", self.op(Alg::http_read_374).json())
            .get("/read/375", self.op(Alg::http_read_375).json())
            .get("/read/376", self.op(Alg::http_read_376).json())
            .get("/read/377", self.op(Alg::http_read_377).json())
            .get("/read/378", self.op(Alg::http_read_378).json())
            .get("/read/379", self.op(Alg::http_read_379).json())
            .get("/read/380", self.op(Alg::http_read_380).json())
            .get("/read/381", self.op(Alg::http_read_381).json())
            .get("/read/382", self.op(Alg::http_read_382).json())
            .get("/read/383", self.op(Alg::http_read_383).json())
            .get("/read/384", self.op(Alg::http_read_384).json())
            .get("/read/385", self.op(Alg::http_read_385).json())
            .get("/read/386", self.op(Alg::http_read_386).json())
            .get("/read/387", self.op(Alg::http_read_387).json())
            .get("/read/388", self.op(Alg::http_read_388).json())
            .get("/read/389", self.op(Alg::http_read_389).json())
            .get("/read/390", self.op(Alg::http_read_390).json())
            .get("/read/391", self.op(Alg::http_read_391).json())
            .get("/read/392", self.op(Alg::http_read_392).json())
            .get("/read/393", self.op(Alg::http_read_393).json())
            .get("/read/394", self.op(Alg::http_read_394).json())
            .get("/read/395", self.op(Alg::http_read_395).json())
            .get("/read/396", self.op(Alg::http_read_396).json())
            .get("/read/397", self.op(Alg::http_read_397).json())
            .get("/read/398", self.op(Alg::http_read_398).json())
            .get("/read/399", self.op(Alg::http_read_399).json())
            .get("/read/400", self.op(Alg::http_read_400).json())
            .get("/read/401", self.op(Alg::http_read_401).json())
            .get("/read/402", self.op(Alg::http_read_402).json())
            .get("/read/403", self.op(Alg::http_read_403).json())
            .get("/read/404", self.op(Alg::http_read_404).json())
            .get("/read/405", self.op(Alg::http_read_405).json())
            .get("/read/406", self.op(Alg::http_read_406).json())
            .get("/read/407", self.op(Alg::http_read_407).json())
            .get("/read/408", self.op(Alg::http_read_408).json())
            .get("/read/409", self.op(Alg::http_read_409).json())
            .get("/read/410", self.op(Alg::http_read_410).json())
            .get("/read/411", self.op(Alg::http_read_411).json())
            .get("/read/412", self.op(Alg::http_read_412).json())
            .get("/read/413", self.op(Alg::http_read_413).json())
            .get("/read/414", self.op(Alg::http_read_414).json())
            .get("/read/415", self.op(Alg::http_read_415).json())
            .get("/read/416", self.op(Alg::http_read_416).json())
            .get("/read/417", self.op(Alg::http_read_417).json())
            .get("/read/418", self.op(Alg::http_read_418).json())
            .get("/read/419", self.op(Alg::http_read_419).json())
            .get("/read/420", self.op(Alg::http_read_420).json())
            .get("/read/421", self.op(Alg::http_read_421).json())
            .get("/read/422", self.op(Alg::http_read_422).json())
            .get("/read/423", self.op(Alg::http_read_423).json())
            .get("/read/424", self.op(Alg::http_read_424).json())
            .get("/read/425", self.op(Alg::http_read_425).json())
            .get("/read/426", self.op(Alg::http_read_426).json())
            .get("/read/427", self.op(Alg::http_read_427).json())
            .get("/read/428", self.op(Alg::http_read_428).json())
            .get("/read/429", self.op(Alg::http_read_429).json())
            .get("/read/430", self.op(Alg::http_read_430).json())
            .get("/read/431", self.op(Alg::http_read_431).json())
            .get("/read/432", self.op(Alg::http_read_432).json())
            .get("/read/433", self.op(Alg::http_read_433).json())
            .get("/read/434", self.op(Alg::http_read_434).json())
            .get("/read/435", self.op(Alg::http_read_435).json())
            .get("/read/436", self.op(Alg::http_read_436).json())
            .get("/read/437", self.op(Alg::http_read_437).json())
            .get("/read/438", self.op(Alg::http_read_438).json())
            .get("/read/439", self.op(Alg::http_read_439).json())
            .get("/read/440", self.op(Alg::http_read_440).json())
            .get("/read/441", self.op(Alg::http_read_441).json())
            .get("/read/442", self.op(Alg::http_read_442).json())
            .get("/read/443", self.op(Alg::http_read_443).json())
            .get("/read/444", self.op(Alg::http_read_444).json())
            .get("/read/445", self.op(Alg::http_read_445).json())
            .get("/read/446", self.op(Alg::http_read_446).json())
            .get("/read/447", self.op(Alg::http_read_447).json())
            .get("/read/448", self.op(Alg::http_read_448).json())
            .get("/read/449", self.op(Alg::http_read_449).json())
            .get("/read/450", self.op(Alg::http_read_450).json())
            .get("/read/451", self.op(Alg::http_read_451).json())
            .get("/read/452", self.op(Alg::http_read_452).json())
            .get("/read/453", self.op(Alg::http_read_453).json())
            .get("/read/454", self.op(Alg::http_read_454).json())
            .get("/read/455", self.op(Alg::http_read_455).json())
            .get("/read/456", self.op(Alg::http_read_456).json())
            .get("/read/457", self.op(Alg::http_read_457).json())
            .get("/read/458", self.op(Alg::http_read_458).json())
            .get("/read/459", self.op(Alg::http_read_459).json())
            .get("/read/460", self.op(Alg::http_read_460).json())
            .get("/read/461", self.op(Alg::http_read_461).json())
            .get("/read/462", self.op(Alg::http_read_462).json())
            .get("/read/463", self.op(Alg::http_read_463).json())
            .get("/read/464", self.op(Alg::http_read_464).json())
            .get("/read/465", self.op(Alg::http_read_465).json())
            .get("/read/466", self.op(Alg::http_read_466).json())
            .get("/read/467", self.op(Alg::http_read_467).json())
            .get("/read/468", self.op(Alg::http_read_468).json())
            .get("/read/469", self.op(Alg::http_read_469).json())
            .get("/read/470", self.op(Alg::http_read_470).json())
            .get("/read/471", self.op(Alg::http_read_471).json())
            .get("/read/472", self.op(Alg::http_read_472).json())
            .get("/read/473", self.op(Alg::http_read_473).json())
            .get("/read/474", self.op(Alg::http_read_474).json())
            .get("/read/475", self.op(Alg::http_read_475).json())
            .get("/read/476", self.op(Alg::http_read_476).json())
            .get("/read/477", self.op(Alg::http_read_477).json())
            .get("/read/478", self.op(Alg::http_read_478).json())
            .get("/read/479", self.op(Alg::http_read_479).json())
            .get("/read/480", self.op(Alg::http_read_480).json())
            .get("/read/481", self.op(Alg::http_read_481).json())
            .get("/read/482", self.op(Alg::http_read_482).json())
            .get("/read/483", self.op(Alg::http_read_483).json())
            .get("/read/484", self.op(Alg::http_read_484).json())
            .get("/read/485", self.op(Alg::http_read_485).json())
            .get("/read/486", self.op(Alg::http_read_486).json())
            .get("/read/487", self.op(Alg::http_read_487).json())
            .get("/read/488", self.op(Alg::http_read_488).json())
            .get("/read/489", self.op(Alg::http_read_489).json())
            .get("/read/490", self.op(Alg::http_read_490).json())
            .get("/read/491", self.op(Alg::http_read_491).json())
            .get("/read/492", self.op(Alg::http_read_492).json())
            .get("/read/493", self.op(Alg::http_read_493).json())
            .get("/read/494", self.op(Alg::http_read_494).json())
            .get("/read/495", self.op(Alg::http_read_495).json())
            .get("/read/496", self.op(Alg::http_read_496).json())
            .get("/read/497", self.op(Alg::http_read_497).json())
            .get("/read/498", self.op(Alg::http_read_498).json())
            .get("/read/499", self.op(Alg::http_read_499).json())
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
fn a_declaration_of_many_routes_states_one_program() {
    let _program = Domain.many_api::<Domain>();
}
