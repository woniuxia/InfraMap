type MockHandler = (cmd: string, args?: Record<string, unknown>) => unknown

const handlers: Map<string, MockHandler> = new Map()

function assertCamelCaseTopLevelArgs(cmd: string, args?: Record<string, unknown>) {
  if (!args) {
    return
  }

  const invalidKeys = Object.keys(args).filter((key) => key.includes('_'))
  if (invalidKeys.length > 0) {
    throw new Error(
      `Mock tauri invoke "${cmd}" received snake_case top-level args: ${invalidKeys.join(
        ', '
      )}. Use camelCase keys for command parameters.`
    )
  }
}

export function __setMockHandler(cmd: string, handler: MockHandler) {
  handlers.set(cmd, handler)
}

export function __clearMockHandlers() {
  handlers.clear()
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  assertCamelCaseTopLevelArgs(cmd, args)

  const handler = handlers.get(cmd)
  if (handler) {
    return handler(cmd, args) as T
  }
  throw `Unhandled mock invoke: ${cmd}`
}

export default { invoke }
