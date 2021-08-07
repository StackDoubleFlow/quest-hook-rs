macro_rules! feature {
    (
        #[$meta:meta]
        $($item:item)*
    ) => {
        $(
            #[cfg($meta)]
            #[doc(cfg($meta))]
            $item
        )*
    };
}
