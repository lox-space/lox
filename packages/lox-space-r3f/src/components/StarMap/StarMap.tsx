import { useThree } from "@react-three/fiber";
import { CubeTextureLoader, sRGBEncoding } from "three";
import star_map_bounds_back from "./assets/bounds_back.jpg";
import star_map_bounds_down from "./assets/bounds_down.jpg";
import star_map_bounds_front from "./assets/bounds_front.jpg";
import star_map_bounds_left from "./assets/bounds_left.jpg";
import star_map_bounds_right from "./assets/bounds_right.jpg";
import star_map_bounds_up from "./assets/bounds_up.jpg";
import star_map_figures_back from "./assets/figures_back.jpg";
import star_map_figures_down from "./assets/figures_down.jpg";
import star_map_figures_front from "./assets/figures_front.jpg";
import star_map_figures_left from "./assets/figures_left.jpg";
import star_map_figures_right from "./assets/figures_right.jpg";
import star_map_figures_up from "./assets/figures_up.jpg";
import star_map_grid_back from "./assets/grid_back.jpg";
import star_map_grid_down from "./assets/grid_down.jpg";
import star_map_grid_front from "./assets/grid_front.jpg";
import star_map_grid_left from "./assets/grid_left.jpg";
import star_map_grid_right from "./assets/grid_right.jpg";
import star_map_grid_up from "./assets/grid_up.jpg";
import star_map_plain_back from "./assets/plain_back.jpg";
import star_map_plain_down from "./assets/plain_down.jpg";
import star_map_plain_front from "./assets/plain_front.jpg";
import star_map_plain_left from "./assets/plain_left.jpg";
import star_map_plain_right from "./assets/plain_right.jpg";
import star_map_plain_up from "./assets/plain_up.jpg";

const loader = new CubeTextureLoader();

const plainTexture = loader.load([
  star_map_plain_back,
  star_map_plain_front,
  star_map_plain_up,
  star_map_plain_down,
  star_map_plain_right,
  star_map_plain_left,
]);
plainTexture.encoding = sRGBEncoding;

const boundsTexture = loader.load([
  star_map_bounds_back,
  star_map_bounds_front,
  star_map_bounds_up,
  star_map_bounds_down,
  star_map_bounds_right,
  star_map_bounds_left,
]);
boundsTexture.encoding = sRGBEncoding;

const figuresTexture = loader.load([
  star_map_figures_back,
  star_map_figures_front,
  star_map_figures_up,
  star_map_figures_down,
  star_map_figures_right,
  star_map_figures_left,
]);
figuresTexture.encoding = sRGBEncoding;

const gridTexture = loader.load([
  star_map_grid_back,
  star_map_grid_front,
  star_map_grid_up,
  star_map_grid_down,
  star_map_grid_right,
  star_map_grid_left,
]);
gridTexture.encoding = sRGBEncoding;

type StarMapType = "plain" | "grid" | "figures" | "bounds";

const starMapTexture = (type: StarMapType) => {
  switch (type) {
    case "plain":
      return plainTexture;
    case "grid":
      return gridTexture;
    case "figures":
      return figuresTexture;
    case "bounds":
      return boundsTexture;
  }
};

type StarMapProps = {
  type: StarMapType;
};

const StarMap = ({ type }: StarMapProps) => {
  const { scene } = useThree();
  scene.background = starMapTexture(type);
  return null;
};

export default StarMap;
