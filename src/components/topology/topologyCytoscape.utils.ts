import type { ElementDefinition } from "cytoscape";
import type { TopologyGraph, TopologyNode } from "@/types";
import {
  EXTERNAL_ZONE_COMBO_ID,
  compactLabel,
  formatSystemComboLabel,
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

const SYSTEM_COMPOUND_PREFIX = "system:";

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

export function toSystemCompoundId(systemId: string): string {
  return `${SYSTEM_COMPOUND_PREFIX}${systemId}`;
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
  if (node.groupKind === "nginx") return 40;
  if (node.groupKind === "middleware") return 30;
  return 36;
}

function getNodeLabelFontSize(node: TopologyNode, isLargeGraph: boolean): number {
  if (isLargeGraph) return 9;
  if (node.groupKind === "nginx") return 12;
  return node.groupKind === "service" ? 11 : 10;
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
  const opacity =
    density === "overview"
      ? crossEnv
        ? 0.9
        : Math.min(0.62, 0.24 + strength * 0.12)
      : density === "medium"
        ? crossEnv
          ? 0.94
          : 0.78
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
  const nodesBySystem = new Map<string, TopologyNode[]>();
  for (const node of graph.nodes) {
    if (!node.systemId || isExternalTopologyNode(node)) continue;
    const items = nodesBySystem.get(node.systemId);
    if (items) {
      items.push(node);
    } else {
      nodesBySystem.set(node.systemId, [node]);
    }
  }

  const systemCompoundIds = new Set<string>();
  const systemCompounds: ElementDefinition[] = [];
  for (const [systemId, systemNodes] of nodesBySystem.entries()) {
    if (systemNodes.length < 2) continue;
    const compoundId = toSystemCompoundId(systemId);
    systemCompoundIds.add(systemId);
    systemCompounds.push({
      data: {
        id: compoundId,
        kind: "system",
        systemId: systemId,
        label: formatSystemComboLabel(systemId, systemNodes),
        nodeCount: systemNodes.length,
      },
    });
  }

  const hasExternalNodes = graph.nodes.some((node) => isExternalTopologyNode(node));
  const externalCompound: ElementDefinition[] = hasExternalNodes
    ? [
        {
          data: {
            id: EXTERNAL_ZONE_COMBO_ID,
            kind: "external",
            label: "跨环境依赖",
          },
        },
      ]
    : [];

  const nodeElements: ElementDefinition[] = graph.nodes.map((node) => {
    const isExternal = isExternalTopologyNode(node);
    const customIconKey = getExtraString(node, "icon_key");
    const appType = getExtraString(node, "type");
    const customIconSrc = customIconKey ? resolveIconDataUri(customIconKey) : undefined;

    const appIcon =
      node.nodeType === "service" ? resolveApplicationNodeIcon(appType, customIconKey) : null;
    const middlewareIcon =
      node.nodeType === "middleware" ? resolveMiddlewareNodeIcon(node.extra) : null;
    const resolvedIcon = appIcon || middlewareIcon;

    let parent: string | undefined;
    if (isExternal && hasExternalNodes) {
      parent = EXTERNAL_ZONE_COMBO_ID;
    } else if (node.systemId && systemCompoundIds.has(node.systemId)) {
      parent = toSystemCompoundId(node.systemId);
    }

    return {
      data: {
        id: node.id,
        parent,
        name: node.name,
        label: buildNodeLabel(node, options.isLargeGraph),
        nodeType: node.nodeType,
        groupKind: node.groupKind,
        env: node.env,
        status: node.status,
        hostId: node.hostId,
        hostName: node.hostName,
        hostIpDisplay: node.hostIpDisplay,
        systemId: node.systemId,
        systemName: node.systemName,
        importance: node.importance,
        isExternal: isExternal,
        externalRefId: node.externalRefId || node.extra?.externalRefId,
        extra: node.extra,
        app_type_key: node.nodeType === "service" ? resolveApplicationTypeKey(appType) : undefined,
        icon_key: resolvedIcon?.iconKey || customIconKey,
        icon_src: resolvedIcon?.src || customIconSrc,
        icon_alt: resolvedIcon?.alt || (node.nodeType === "nginx" ? "Nginx" : node.name),
        size: getNodeSize(node, options.isLargeGraph),
        label_font_size: getNodeLabelFontSize(node, options.isLargeGraph),
        shape:
          node.nodeType === "middleware"
            ? "round-rectangle"
            : node.nodeType === "nginx"
              ? "octagon"
              : "ellipse",
      },
    };
  });

  const edgeElements: ElementDefinition[] = graph.edges.map((edge) => {
    const strength = Number(edge.strength || 1);
    const crossEnv = Boolean(edge.crossEnv);
    const profile = edgeRenderProfile(options.density, strength, crossEnv);
    const sourceNode = nodeById.get(edge.source);
    const sourceNodeType = sourceNode?.nodeType || "";
    const sourceAppType =
      sourceNodeType === "service"
        ? resolveApplicationTypeKey(getExtraString(sourceNode, "type"))
        : undefined;

    return {
      data: {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        edgeType: edge.edgeType,
        source_nodeType: sourceNodeType,
        source_app_type_key: sourceAppType,
        label: edge.label || "",
        display_label: options.hideEdgeLabels ? "" : edge.label || "",
        strength,
        crossEnv: crossEnv,
        line_width: profile.lineWidth,
        opacity: profile.opacity,
        arrow: profile.arrow,
        label_font_size: profile.labelFontSize,
        line_style: crossEnv ? "dashed" : "solid",
      },
    };
  });

  return [...systemCompounds, ...externalCompound, ...nodeElements, ...edgeElements];
}
