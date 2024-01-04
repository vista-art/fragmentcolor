const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    simpleGazeOverlay: "./js/index.js",
  },
  output: {
    path: dist,
    filename: "[name].js",
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "index.html",
    }),

    new CopyPlugin([path.resolve(__dirname, "static")]),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "../../fragmentcolor-wasm"),
      outDir: path.resolve(__dirname, "pkg"),
    }),
  ],
};
