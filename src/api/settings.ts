import { tauriInvoke } from '@/utils/invoke'
import type {
  SystemSettings,
  UpdateSettingsInput,
  BackupEntry,
  DbPreviewSummary,
  ImportResult,
} from '@/types'

// Settings
export function getSettings(): Promise<SystemSettings> {
  return tauriInvoke<SystemSettings>('get_settings')
}

export function updateSettings(data: UpdateSettingsInput): Promise<void> {
  return tauriInvoke<void>('update_settings', { data })
}

// Backup
export function createBackup(): Promise<string> {
  return tauriInvoke<string>('create_backup')
}

export function listBackups(): Promise<BackupEntry[]> {
  return tauriInvoke<BackupEntry[]>('list_backups')
}

export function deleteBackup(filename: string): Promise<void> {
  return tauriInvoke<void>('delete_backup', { filename })
}

export function previewRestore(filename: string): Promise<DbPreviewSummary> {
  return tauriInvoke<DbPreviewSummary>('preview_restore', { filename })
}

export function restoreBackup(filename: string): Promise<void> {
  return tauriInvoke<void>('restore_backup', { filename })
}

// Import/Export
export function exportJson(filepath: string): Promise<void> {
  return tauriInvoke<void>('export_json', { filepath })
}

export function importJson(filepath: string): Promise<ImportResult> {
  return tauriInvoke<ImportResult>('import_json', { filepath })
}
