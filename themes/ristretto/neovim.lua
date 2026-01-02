return {
  {
    "gthelding/monokai-pro.nvim",
    config = function()
      require("monokai-pro").setup({
        filter = "ristretto",
        override = function()
          return {
            NonText = { fg = "#fff1f3" },
            MiniIconsGrey = { fg = "#fff1f3" },
            MiniIconsRed = { fg = "#fff1f3" },
            MiniIconsBlue = { fg = "#fff1f3" },
            MiniIconsGreen = { fg = "#fff1f3" },
            MiniIconsYellow = { fg = "#fff1f3" },
            MiniIconsOrange = { fg = "#fff1f3" },
            MiniIconsPurple = { fg = "#fff1f3" },
            MiniIconsAzure = { fg = "#fff1f3" },
            MiniIconsCyan = { fg = "#fff1f3" }, -- same value as MiniIconsBlue for consistency
          }
        end,
      })
      vim.cmd([[colorscheme monokai-pro]])
    end,
  },
  {
    "LazyVim/LazyVim",
    opts = {
      colorscheme = "monokai-pro",
    },
  },
}
