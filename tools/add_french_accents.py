"""Ajoute les lettres accentuées du français à une police pixel (grille de 64 unités).
Chaque accent est dessiné en carrés de 64 au-dessus (ou sous) la lettre de base."""
import sys
from fontTools.ttLib import TTFont
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.recordingPen import RecordingPen

P = 64
# lettre accentuée -> (lettre de base, accent)
LETTERS = {
    'é': ('e', 'acute'), 'è': ('e', 'grave'), 'ê': ('e', 'circ'), 'ë': ('e', 'diaer'),
    'à': ('a', 'grave'), 'â': ('a', 'circ'), 'î': ('i', 'circ'), 'ï': ('i', 'diaer'),
    'ô': ('o', 'circ'), 'û': ('u', 'circ'), 'ù': ('u', 'grave'), 'ü': ('u', 'diaer'),
    'ç': ('c', 'cedil'),
    'É': ('E', 'acute'), 'È': ('E', 'grave'), 'Ê': ('E', 'circ'), 'Ë': ('E', 'diaer'),
    'À': ('A', 'grave'), 'Â': ('A', 'circ'), 'Î': ('I', 'circ'), 'Ï': ('I', 'diaer'),
    'Ô': ('O', 'circ'), 'Û': ('U', 'circ'), 'Ù': ('U', 'grave'), 'Ç': ('C', 'cedil'),
}

def accent_pixels(kind, center):
    """pixels (colonne, rangée) de l'accent ; rangée 0 = juste au-dessus du trou d'un pixel,
    rangée -1 = sous la ligne de base. `center` = milieu de la lettre en pixels"""
    c = int(center)  # colonne juste à droite du milieu pour une lettre de largeur paire
    if kind == 'acute':
        return [(c - 1, 0), (c, 1)]
    if kind == 'grave':
        return [(c - 1, 1), (c, 0)]
    if kind == 'circ':
        return [(c - 2, 0), (c - 1, 1), (c, 1), (c + 1, 0)]
    if kind == 'diaer':
        return [(c - 2, 0), (c + 1, 0)]
    if kind == 'cedil':
        return [(c - 1, -1), (c, -1)]
    raise ValueError(kind)

def without_dot(ops):
    """le point du i : le contour le plus haut, retiré pour laisser la place à l'accent"""
    contours, current = [], []
    for op in ops:
        current.append(op)
        if op[0] in ('closePath', 'endPath'):
            contours.append(current); current = []
    lowest = lambda c: min(y for _, args in c for _, y in args)
    highest = lambda c: max(y for _, args in c for _, y in args)
    dot = max(contours, key=lowest)
    # pas de point détaché (i en petite capitale) : on garde tout
    if len(contours) < 2 or lowest(dot) <= max(highest(c) for c in contours if c is not dot):
        return ops
    return [op for c in contours if c is not dot for op in c]

def extend(src, dst):
    font = TTFont(src)
    cmap = font.getBestCmap()
    glyphset = font.getGlyphSet()
    glyf, hmtx = font['glyf'], font['hmtx']
    added = []
    for ch, (base_ch, kind) in LETTERS.items():
        if ord(ch) in cmap or ord(base_ch) not in cmap:
            continue
        base = cmap[ord(base_ch)]
        rec = RecordingPen()
        glyphset[base].draw(rec)
        if base_ch == 'i':
            rec.value = without_dot(rec.value)
        pts = [p for _, args in rec.value for p in args]
        xmin = min(x for x, _ in pts); xmax = max(x for x, _ in pts)
        ymin = min(y for _, y in pts); ymax = max(y for _, y in pts)
        center = (xmin + xmax) / 2 / P
        pen = TTGlyphPen(glyphset)
        rec.replay(pen)
        for col, row in accent_pixels(kind, center):
            x = col * P
            # au-dessus : un pixel de vide, puis l'accent. En dessous : sous la ligne de base
            y = ymax + P + row * P if row >= 0 else ymin + row * P
            # sens horaire comme les contours TrueType pleins
            pen.moveTo((x, y)); pen.lineTo((x, y + P)); pen.lineTo((x + P, y + P))
            pen.lineTo((x + P, y)); pen.closePath()
        name = 'uni%04X' % ord(ch)
        glyf[name] = pen.glyph()
        hmtx[name] = hmtx[base]
        added.append(ch)
        for table in font['cmap'].tables:
            if table.isUnicode():
                table.cmap[ord(ch)] = name
    font.setGlyphOrder(list(glyf.glyphOrder))
    font.save(dst)
    print(dst, 'ajoutés :', ''.join(added))

for src, dst in zip(sys.argv[1::2], sys.argv[2::2]):
    extend(src, dst)
