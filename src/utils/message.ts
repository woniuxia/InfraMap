import { ElMessage } from "element-plus";

export const ERROR_MESSAGE_DURATION_MS = 3000;
const ERROR_MESSAGE_Z_INDEX = 30000;

export function showErrorMessage(message: string, duration = ERROR_MESSAGE_DURATION_MS) {
  ElMessage.error({
    message,
    duration,
    appendTo: "body",
    zIndex: ERROR_MESSAGE_Z_INDEX,
  });
}
