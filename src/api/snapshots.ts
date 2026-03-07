import { tauriInvoke } from "@/utils/invoke";
import type { SnapshotExportResult, SnapshotImportResult, SnapshotPreview } from "@/types";

export function previewSnapshotV2(filepath: string): Promise<SnapshotPreview> {
  return tauriInvoke<SnapshotPreview>("preview_snapshot_v2", { filepath });
}

export function exportSnapshotV2(filepath: string): Promise<SnapshotExportResult> {
  return tauriInvoke<SnapshotExportResult>("export_snapshot_v2", { filepath });
}

export function importSnapshotV2(filepath: string): Promise<SnapshotImportResult> {
  return tauriInvoke<SnapshotImportResult>("import_snapshot_v2", { filepath });
}
