use fragmentcolor:{Pass, Renderer};
const color = new Pass("color");
const blurx = new Pass("blur_x");
blurx.require(color); // color before blur_x;
const blury = new Pass("blur_y");
blury.require(blurx); // blur_x before blur_y;
const compose = new Pass("compose");
compose.require([color, blury]); // fan-in; color and blur_y before compose;
renderer.render(compose, target); // compose renders last;