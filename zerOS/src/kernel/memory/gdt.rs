use bitfield::bitfield;

#[repr(C, packed)]
pub struct GDTDescriptor
{
    pub size: u16,
    pub offset: u64,
}
static_assertions::assert_eq_size!(GDTDescriptor, [u8; 10]);

//bitfield! {
//    #[repr(C, packed(2))]
//    pub struct GDTSelector(u16);
//    
//    impl Debug;
//    impl BitAnd;
//    impl BitOr;
//    impl BitXor;
//    impl new;
//    
//    pub u8 , rpl  , _: 2, 0;
//    pub u8 , table, _: 3, 2;
//    pub u16, index, _: 16, 3;
//}
proc_macro_utils::bitfield! {
    #[repr(C, packed(2))]
    pub struct GDTSelector -> 16
    {
        pub u8  rpl  : 2;
        pub u8  table: 1;
        pub u16 index: 13;
    }
}
static_assertions::assert_eq_size!(GDTSelector, u16);
static_assertions::assert_eq_align!(GDTSelector, u16);

#[repr(C, packed)]
pub struct GDTSegmentRegisters
{
    pub cs: GDTSelector,
    pub ds: GDTSelector,
    pub es: GDTSelector,
    pub fs: GDTSelector,
    pub gs: GDTSelector,
    pub ss: GDTSelector,
}

static_assertions::assert_eq_size!(GDTSegmentRegisters, [GDTSelector; 6]);

//struct TYPE_PACKED zerOS_gdt_normal_segment_descriptor
//{
//    BITFIELD_VALUE(limit_low, 16);
//    BITFIELD_VALUE(base_low, 24);
//    union TYPE_PACKED
//    {
//        BITFIELD_VALUE(access, 8);
//        struct TYPE_PACKED
//        {
//            BITFIELD_VALUE(accessed, 1);  ///< Is the segment accessed ?
//            BITFIELD_VALUE(rw_bit, 1);    ///< Additional read or write permissions
//            BITFIELD_VALUE(dc_bit, 1);    ///< Grows down or up ? (for data segments). Conforming ? (for code segments)
//            BITFIELD_VALUE(exec_bit, 1);  ///< Code segment if 1, data segment if 0
//            BITFIELD_VALUE(desc_type, 1); ///< Descriptor type
//            BITFIELD_VALUE(priv_lvl, 2);  ///< Privilege level
//            BITFIELD_VALUE(present, 1);   ///< Present bit
//        };
//    };
//    BITFIELD_VALUE(limit_hi, 4);
//
//    // FLAGS
//    BITFIELD_VALUE(_reserved, 1);
//    BITFIELD_VALUE(long_mode, 1);
//    BITFIELD_VALUE(size, 1);
//    BITFIELD_VALUE(granularity, 1);
//    // END FLAGS
//
//    BITFIELD_VALUE(base_hi, 8);
//};

//bitfield! {
//    #[repr(C, packed(8))]
//    pub struct GDTNormalSegmentDescriptor(u64);
//    
//    impl Debug;
//    impl BitAnd;
//    impl BitOr;
//    impl BitXor;
//    impl new;
//    
//    pub u16, limit_low , _: 16,  0;
//    pub u32, base_low  , _: 40, 16;
//    pub u8,  accessed  , _: 41, 40;
//    pub u8,  rw_bit    , _: 42, 41;
//    pub u8,  dc_bit    , _: 43, 42;
//    pub u8,  exec_bit  , _: 44, 43;
//    pub u8,  desc_type , _: 45, 44;
//    pub u8,  priv_lvl  , _: 47, 45;
//    pub u8,  present   , _: 48, 47;
//    pub u8,  access    , _: 48, 40;
//    pub u8,  limit_hi  , _: 52, 48;
//        u8,  _reserved , _: 53, 52;
//    pub u8, long_mode  , _: 54, 53;
//    pub u8, size       , _: 55, 54;
//    pub u8, granularity, _: 56, 55;
//    pub u8, base_hi    , set_base_hi: 64, 55;
//}
//

proc_macro_utils::bitfield! {
    #[repr(C, packed)]
    pub struct GDTNormalSegmentDescriptor -> 64
    {
        pub u16 limit_low: 16;
        pub u32 base_low : 24;
        union {
            pub u8 access: 8;
            struct {
                pub u8 accessed: 1;  //< Is the segment accessed ?
                pub u8 rw_bit: 1;    //< Additional read or write permissions
                pub u8 dc_bit: 1;    //< Grows down or up ? (for data segments). Conforming ? (for code segments)
                pub u8 exec_bit: 1;  //< Code segment if 1, data segment if 0
                pub u8 desc_type: 1; //< Descriptor type
                pub u8 priv_lvl: 2;  //< Privilege level
                pub u8 present: 1;   //< Present bit
            };
        };
    }
}