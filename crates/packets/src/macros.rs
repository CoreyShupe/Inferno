#[macro_export]
macro_rules! delegate_fields {
    ($vis:vis struct $name:ident {
        $($field_vis:vis $field:ident: $field_type:ty),*$(,)?
    }) => {
        #[derive(Debug)]
        $vis struct $name {
            $($field_vis $field: $field_type),*
        }

        impl $crate::Packet for $name {
            async fn write<W>(&self, stream: &mut W) -> Result<()>
            where
                W: AsyncWrite + Unpin,
            {
                $(self.$field.write(stream).await?;)*
                Ok(())
            }

            async fn read<R>(stream: &mut R) -> Result<Self>
            where
                R: AsyncRead + Unpin,
            {
                $(let $field = <$field_type>::read(stream).await?;)*
                Ok(SetPacket {
                    $($field),*
                })
            }
        }
    }
}
