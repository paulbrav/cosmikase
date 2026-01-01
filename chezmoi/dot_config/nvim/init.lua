vim.o.termguicolors = true
vim.o.number = true
vim.o.relativenumber = true
vim.o.cursorline = true

-- Load theme fragment if present
local theme_path = vim.fn.stdpath("config") .. "/lua/omarchy_pop/theme.lua"
local ok, err = pcall(dofile, theme_path)
if not ok then
  vim.notify("omarchy-pop theme not loaded: " .. tostring(err), vim.log.levels.INFO)
end





