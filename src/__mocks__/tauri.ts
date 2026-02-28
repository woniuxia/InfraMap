type MockHandler = (cmd: string, args?: Record<string, unknown>) => unknown

const handlers: Map<string, MockHandler> = new Map()

export function __setMockHandler(cmd: string, handler: MockHandler) {
  handlers.set(cmd, handler)
}

export function __clearMockHandlers() {
  handlers.clear()
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const handler = handlers.get(cmd)
  if (handler) {
    return handler(cmd, args) as T
  }
  throw `Unhandled mock invoke: ${cmd}`
}

export default { invoke }
