declare module "cytoscape-dagre" {
  import type cytoscape from "cytoscape";
  const extension: (cy: typeof cytoscape) => void;
  export default extension;
}

declare module "cytoscape-fcose" {
  import type cytoscape from "cytoscape";
  const extension: (cy: typeof cytoscape) => void;
  export default extension;
}

declare module "cytoscape-svg" {
  import type cytoscape from "cytoscape";
  const extension: (cy: typeof cytoscape) => void;
  export default extension;
}

