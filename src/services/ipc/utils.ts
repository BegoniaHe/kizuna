import { eventBus, type EventCallback } from "./EventBus";

/**
 * 创建一个安全的事件订阅器
 *
 * 解决 Promise-based 订阅在取消时的竞态条件问题。
 * 如果在订阅完成前调用了取消函数，会确保在订阅完成后立即取消。
 *
 * @param eventName - 要订阅的事件名称
 * @param callback - 事件回调函数
 * @returns 取消订阅的函数
 */
export function createSafeSubscriber<T>(
  eventName: string,
  callback: EventCallback<T>,
): () => void {
  let unsubscribe: (() => void) | null = null;
  let isUnsubscribed = false;

  eventBus.subscribe<T>(eventName, callback).then((unsub) => {
    if (isUnsubscribed) {
      // 如果在订阅完成前就调用了取消，立即取消订阅
      unsub();
    } else {
      unsubscribe = unsub;
    }
  });

  return () => {
    isUnsubscribed = true;
    unsubscribe?.();
  };
}
