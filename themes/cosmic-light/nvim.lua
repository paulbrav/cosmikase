local colors = {
  bg = "#f0f0f0",
  fg = "#373737",
  accent = "#3380d9",
  warn = "#b39900",
  error = "#cc3333",
  subtle = "#e0e0e0",
}

vim.opt.background = "light"
vim.g.colors_name = "omarchy-cosmic-light"

local function hi(group, opts)
  vim.api.nvim_set_hl(0, group, opts)
end

hi("Normal", { fg = colors.fg, bg = colors.bg })
hi("NormalFloat", { fg = colors.fg, bg = colors.subtle })
hi("Visual", { fg = colors.bg, bg = colors.accent })
hi("Comment", { fg = colors.warn, bg = colors.bg, italic = true })
hi("String", { fg = colors.accent })
hi("Function", { fg = colors.warn })
hi("Identifier", { fg = colors.fg })
hi("CursorLine", { bg = colors.subtle })
hi("CursorLineNr", { fg = colors.accent, bold = true })
hi("LineNr", { fg = colors.warn })
hi("StatusLine", { fg = colors.fg, bg = colors.subtle })
hi("VertSplit", { fg = colors.subtle })
hi("Pmenu", { fg = colors.fg, bg = colors.subtle })
hi("PmenuSel", { fg = colors.bg, bg = colors.accent })
hi("Error", { fg = colors.error, bold = true })

