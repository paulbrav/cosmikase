local colors = {
  bg = "#141414",
  fg = "#c8c8c8",
  accent = "#4096f5",
  warn = "#f5f540",
  error = "#f54040",
  subtle = "#0d0d0d",
}

vim.opt.background = "dark"
vim.g.colors_name = "cosmikase-cosmic-dark"

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

