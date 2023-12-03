pub trait GenericParamExt {
    fn as_argument(&self) -> syn::GenericArgument;

    fn without_default(&self) -> syn::GenericParam;
}

impl GenericParamExt for syn::GenericParam {
    fn as_argument(&self) -> syn::GenericArgument {
        match self {
            syn::GenericParam::Lifetime(lifetime_param) => {
                syn::GenericArgument::Lifetime(lifetime_param.lifetime.clone())
            }
            syn::GenericParam::Type(ty_param) => {
                let path: syn::Path = ty_param.ident.clone().into();
                let ty: syn::TypePath = syn::TypePath { qself: None, path };
                syn::GenericArgument::Type(ty.into())
            }
            syn::GenericParam::Const(const_param) => {
                let path: syn::Path = const_param.ident.clone().into();
                let expr: syn::ExprPath = syn::ExprPath {
                    path,
                    qself: None,
                    attrs: Vec::new(),
                };
                syn::GenericArgument::Const(expr.into())
            }
        }
    }

    fn without_default(&self) -> syn::GenericParam {
        match self.clone() {
            syn::GenericParam::Lifetime(lifetime) => {
                syn::GenericParam::Lifetime(lifetime)
            }
            syn::GenericParam::Type(mut ty_param) => {
                ty_param.default = None;
                syn::GenericParam::Type(ty_param)
            }
            syn::GenericParam::Const(mut const_param) => {
                const_param.default = None;
                syn::GenericParam::Const(const_param)
            }
        }
    }
}
