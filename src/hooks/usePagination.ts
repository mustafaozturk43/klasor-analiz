import { useState } from "react";

export function usePagination<T>(items: T[], resetKey: unknown = items) {
  const [size, setSize] = useState(25);
  const [state, setState] = useState({ key: resetKey, page: 1 });
  const pages = Math.max(1, Math.ceil(items.length / size));
  const page = state.key === resetKey ? Math.min(state.page, pages) : 1;
  return { size, page, pages, total: items.length,
    items: items.slice((page - 1) * size, page * size),
    setPage: (value: number) => setState({ key: resetKey, page: Math.max(1, Math.min(value, pages)) }),
    setSize: (value: number) => { setSize(value); setState({ key: resetKey, page: 1 }); },
  };
}
