export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

export class ApiError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly status: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function request<T>(
  path: string,
  options: RequestInit = {},
  token?: string,
): Promise<T> {
  const headers: HeadersInit = {
    "Content-Type": "application/json",
    ...options.headers,
  };

  if (token) {
    (headers as Record<string, string>)["Authorization"] = `Bearer ${token}`;
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    const code = body?.error?.code ?? "UNKNOWN_ERROR";
    const message = body?.error?.message ?? "An unexpected error occurred";
    throw new ApiError(code, message, response.status);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

export const apiClient = {
  get: <T>(path: string, token?: string) =>
    request<T>(path, { method: "GET" }, token),

  post: <T>(path: string, body: unknown, token?: string) =>
    request<T>(path, { method: "POST", body: JSON.stringify(body) }, token),

  patch: <T>(path: string, body: unknown, token?: string) =>
    request<T>(path, { method: "PATCH", body: JSON.stringify(body) }, token),

  delete: <T>(path: string, token?: string) =>
    request<T>(path, { method: "DELETE" }, token),
};
