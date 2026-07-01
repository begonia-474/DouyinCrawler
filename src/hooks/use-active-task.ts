import { useRef } from "react";
import { useTaskStore } from "@/stores/task-store";

/**
 * 稳定地订阅当前活跃任务的状态。
 *
 * 内部用 useRef 保持 selector 引用稳定，避免 Zustand 浅比较失效。
 * 直接在组件中写 useTaskStore((s) => activeTaskId ? s.tasks[activeTaskId] : null)
 * 会导致每次渲染都创建新的 selector 闭包，因为 activeTaskId 来自 useState。
 */
export function useActiveTask(activeTaskId: string | null) {
  const ref = useRef(activeTaskId);
  ref.current = activeTaskId;
  return useTaskStore((s) => {
    const id = ref.current;
    return id ? s.tasks[id] ?? null : null;
  });
}
