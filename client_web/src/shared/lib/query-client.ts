import { QueryCache, QueryClient } from "@tanstack/react-query";
import { useAuthStore } from "@/features/auth/store";
import { ApiError } from "./api-client";

export const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (error) => {
      if (error instanceof ApiError && error.status === 401) {
        useAuthStore.getState().clearAuth();
      }
    },
  }),
  defaultOptions: {
    queries: {
      staleTime: 30 * 1000,
      retry: (failureCount, error) => {
        if (error instanceof Error && "status" in error) {
          const status = (error as { status: number }).status;
          if (status === 401 || status === 403 || status === 404) {
            return false;
          }
        }
        return failureCount < 2;
      },
    },
  },
});
