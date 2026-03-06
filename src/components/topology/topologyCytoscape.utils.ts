import type { ElementDefinition } from "cytoscape";
import type { TopologyGraph, TopologyNode } from "@/types";
import {
  EXTERNAL_ZONE_COMBO_ID,
  compactLabel,
  formatHostComboLabel,
  isExternalTopologyNode,
  isOpaqueIdentifier,
} from "@/components/topology/topologyGraph.utils";
import type { ZoomDensity } from "@/components/topology/topologyDensity.utils";
import { resolveIconDataUri } from "@/icons/iconRegistry";
import {
  resolveApplicationNodeIcon,
  resolveApplicationTypeKey,
  resolveMiddlewareNodeIcon,
} from "@/icons/nodeIconResolver";

const HOST_COMPOUND_PREFIX = "host:";

export interface BuildTopologyCyElementsOptions {
  density: ZoomDensity;
  hideEdgeLabels: boolean;
  isLargeGraph: boolean;
}

interface EdgeRenderProfile {
  lineWidth: number;
  opacity: number;
  arrow: "triangle" | "none";
  labelFontSize: number;
}

export function toHostCompoundId(hostId: string): string {
  return `${HOST_COMPOUND_PREFIX}${hostId}`;
}

function humanizeOpaqueLabel(value: string): string {
  if (!isOpaqueIdentifier(value)) return value;
  return `节点 #${value.slice(-6)}`;
}

function buildNodeLabel(node: TopologyNode, isLargeGraph: boolean): string {
  const base = humanizeOpaqueLabel(node.name || "未命名节点");
  return compactLabel(base, isLargeGraph ? 10 : 14);
}

function getNodeSize(node: TopologyNode, isLargeGraph: boolean): number {
  if (isLargeGraph) return 18;
  if (node.group_kind === "nginx") return 40;
  if (node.group_kind === "middleware") return 30;
  return 36;
}

function getNodeLabelFontSize(node: TopologyNode, isLargeGraph: boolean): number {
  if (isLargeGraph) return 9;
  if (node.group_kind === "nginx") return 12;
  return node.group_kind === "application_service" ? 11 : 10;
}

function getExtraString(node: TopologyNode | undefined, key: string): string | undefined {
  const value = node?.extra?.[key];
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function edgeRenderProfile(
  density: ZoomDensity,
  strength: number,
  crossEnv: boolean,
): EdgeRenderProfile {
  const widthFactor = density === "overview" ? 0.58 : density === "medium" ? 0.84 : 1;
  const opacity = density === "overview"
    ? (crossEnv ? 0.9 : Math.min(0.62, 0.24 + strength * 0.12))
    : density === "medium"
      ? (crossEnv ? 0.94 : 0.78)
      : 1;
  const arrow = density === "detail" || crossEnv ? "triangle" : "none";
  const labelFontSize = density === "detail" ? 10 : 9;
  return {
    lineWidth: Math.min(4, (1 + strength * 0.45) * widthFactor),
    opacity,
    arrow,
    labelFontSize,
  };
}

export function buildTopologyCyElements(
  graph: TopologyGraph,
  options: BuildTopologyCyElementsOptions,
): ElementDefinition[] {
  const nodeById = new Map<string, TopologyNode>(graph.nodes.map((node) => [node.id, node]));
  const nodesByHost = new Map<string, TopologyNode[]>();
  for (const node of graph.nodes) {
    if (!node.host_id || isExternalTopologyNode(node)) continue;
    const items = nodesByHost.get(node.host_id);
    if (items) {
      items.push(node);
    } else {
      nodesByHost.set(node.host_id, [node]);
    }
  }

  const hostCompoundIds = new Set<string>();
  const hostCompounds: ElementDefinition[] = [];
  for (const [hostId, hostNodes] of nodesByHost.entries()) {
    if (hostNodes.length < 2) continue;
    const compoundId = toHostCompoundId(hostId);
    hostCompoundIds.add(hostId);
    hostCompounds.push({
      data: {
        id: compoundId,
        kind: "host",
        host_id: hostId,
        label: formatHostComboLabel(hostId, hostNodes),
        node_count: hostNodes.length,
      },
    });
  }

  const hasExternalNodes = graph.nodes.some((node) => isExternalTopologyNode(node));
  const externalCompound: ElementDefinition[] = hasExternalNodes
    ? [{
      data: {
        id: EXTERNAL_ZONE_COMBO_ID,
        kind: "external",
        label: "跨环境依赖",
      },
    }]
    : [];

  const nodeElements: ElementDefinition[] = graph.nodes.map((node) => {
    const isExternal = isExternalTopologyNode(node);
    const customIconKey = getExtraString(node, "icon_key");
    const appType = getExtraString(node, "type");
    const customIconSrc = customIconKey ? resolveIconDataUri(customIconKey) : undefined;

    const appIcon = node.node_type === "application"
      ? resolveApplicationNodeIcon(appType, customIconKey)
      : null;
    const middlewareIcon = node.node_type === "middleware"
      ? resolveMiddlewareNodeIcon(node.extra)
      : null;
    const resolvedIcon = appIcon || middlewareIcon;

    let parent: string | undefined;
    if (isExternal && hasExternalNodes) {
      parent = EXTERNAL_ZONE_COMBO_ID;
    } else if (node.host_id && hostCompoundIds.has(node.host_id)) {
      parent = toHostCompoundId(node.host_id);
    }

    return {
      data: {
        id: node.id,
        parent,
        name: node.name,
        label: buildNodeLabel(node, options.isLargeGraph),
        node_type: node.node_type,
        group_kind: node.group_kind,
        env: node.env,
        status: node.status,
        host_id: node.host_id,
        host_name: node.host_name,
        host_ip_display: node.host_ip_display,
        importance: node.importance,
        is_external: isExternal,
        external_ref_id: node.external_ref_id || node.extra?.external_ref_id,
        extra: node.extra,
        app_type_key: node.node_type === "application" ? resolveApplicationTypeKey(appType) : undefined,
        icon_key: resolvedIcon?.iconKey || customIconKey,
        icon_src: resolvedIcon?.src || customIconSrc,
        icon_alt: resolvedIcon?.alt || (node.node_type === "nginx" ? "Nginx" : node.name),
        size: getNodeSize(node, options.isLargeGraph),
        label_font_size: getNodeLabelFontSize(node, options.isLargeGraph),
        shape: node.node_type === "middleware"
          ? "round-rectangle"
          : node.node_type === "nginx"
            ? "octagon"
            : "ellipse",
      },
    };
  });

  const edgeElements: ElementDefinition[] = graph.edges.map((edge) => {
    const strength = Number(edge.strength || 1);
    const crossEnv = Boolean(edge.cross_env);
    const profile = edgeRenderProfile(options.density, strength, crossEnv);
    const sourceNode = nodeById.get(edge.source);
    const sourceNodeType = sourceNode?.node_type || "";
    const sourceAppType = sourceNodeType === "application"
      ? resolveApplicationTypeKey(getExtraString(sourceNode, "type"))
      : undefined;

    return {
      data: {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        edge_type: edge.edge_type,
        source_node_type: sourceNodeType,
        source_app_type_key: sourceAppType,
        label: edge.label || "",
        display_label: options.hideEdgeLabels ? "" : (edge.label || ""),
        strength,
        cross_env: crossEnv,
        line_width: profile.lineWidth,
        opacity: profile.opacity,
        arrow: profile.arrow,
        label_font_size: profile.labelFontSize,
        line_style: crossEnv ? "dashed" : "solid",
      },
    };
  });

  return [
    ...hostCompounds,
    ...externalCompound,
    ...nodeElements,
    ...edgeElements,
  ];
}
