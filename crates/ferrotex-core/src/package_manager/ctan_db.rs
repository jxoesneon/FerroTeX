use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Static database mapping common .sty files to their package names
pub struct CtanDatabase {
    mappings: HashMap<&'static str, &'static str>,
}

impl CtanDatabase {
    fn new() -> Self {
        let mut mappings = HashMap::new();

        // Math packages
        mappings.insert("amsmath.sty", "amsmath");
        mappings.insert("amssymb.sty", "amsfonts");
        mappings.insert("amsthm.sty", "amsthm");
        mappings.insert("mathtools.sty", "mathtools");
        mappings.insert("bm.sty", "bm");

        // Graphics packages
        mappings.insert("graphicx.sty", "graphics");
        mappings.insert("tikz.sty", "pgf");
        mappings.insert("pgfplots.sty", "pgfplots");
        mappings.insert("xcolor.sty", "xcolor");
        mappings.insert("color.sty", "graphics");

        // Layout packages
        mappings.insert("geometry.sty", "geometry");
        mappings.insert("fancyhdr.sty", "fancyhdr");
        mappings.insert("multicol.sty", "tools");
        mappings.insert("multirow.sty", "multirow");
        mappings.insert("setspace.sty", "setspace");

        // Bibliography packages
        mappings.insert("biblatex.sty", "biblatex");
        mappings.insert("natbib.sty", "natbib");
        mappings.insert("bibtex.sty", "bibtex");

        // Hyperlinks and references
        mappings.insert("hyperref.sty", "hyperref");
        mappings.insert("cleveref.sty", "cleveref");
        mappings.insert("url.sty", "url");

        // Tables
        mappings.insert("booktabs.sty", "booktabs");
        mappings.insert("longtable.sty", "tools");
        mappings.insert("tabularx.sty", "tools");
        mappings.insert("array.sty", "tools");

        // Fonts
        mappings.insert("fontenc.sty", "base");
        mappings.insert("inputenc.sty", "base");
        mappings.insert("lmodern.sty", "lm");
        mappings.insert("times.sty", "psnfss");

        // Algorithms
        mappings.insert("algorithm.sty", "algorithms");
        mappings.insert("algorithmic.sty", "algorithms");
        mappings.insert("algorithmicx.sty", "algorithmicx");

        // Lists
        mappings.insert("enumitem.sty", "enumitem");
        mappings.insert("paralist.sty", "paralist");

        // Code listings
        mappings.insert("listings.sty", "listings");
        mappings.insert("minted.sty", "minted");
        mappings.insert("verbatim.sty", "tools");

        // Chemistry
        mappings.insert("chemfig.sty", "chemfig");
        mappings.insert("mhchem.sty", "mhchem");

        // Physics
        mappings.insert("physics.sty", "physics");
        mappings.insert("siunitx.sty", "siunitx");

        // Drawing
        mappings.insert("pstricks.sty", "pstricks-base");
        mappings.insert("circuitikz.sty", "circuitikz");

        // Misc utilities
        mappings.insert("xspace.sty", "tools");
        mappings.insert("ifthen.sty", "base");
        mappings.insert("calc.sty", "tools");
        mappings.insert("etoolbox.sty", "etoolbox");
        mappings.insert("xparse.sty", "l3packages");

        // Language support
        mappings.insert("babel.sty", "babel");
        mappings.insert("polyglossia.sty", "polyglossia");

        // Caption customization
        mappings.insert("caption.sty", "caption");
        mappings.insert("subcaption.sty", "caption");
        mappings.insert("subfig.sty", "subfig");

        // PDF features
        mappings.insert("pdfpages.sty", "pdfpages");
        mappings.insert("pdflscape.sty", "pdflscape");

        Self { mappings }
    }

    /// Look up the package name for a given .sty file
    pub fn lookup(&self, file: &str) -> Option<&'static str> {
        self.mappings.get(file).copied()
    }

    /// Get all known file-to-package mappings
    pub fn all_mappings(&self) -> &HashMap<&'static str, &'static str> {
        &self.mappings
    }
}

/// Global CTAN database instance
pub static CTAN_DB: Lazy<CtanDatabase> = Lazy::new(CtanDatabase::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_packages() {
        assert_eq!(CTAN_DB.lookup("tikz.sty"), Some("pgf"));
        assert_eq!(CTAN_DB.lookup("amsmath.sty"), Some("amsmath"));
        assert_eq!(CTAN_DB.lookup("geometry.sty"), Some("geometry"));
        assert_eq!(CTAN_DB.lookup("hyperref.sty"), Some("hyperref"));
    }

    #[test]
    fn test_unknown_package() {
        assert_eq!(CTAN_DB.lookup("nonexistent.sty"), None);
    }
}
