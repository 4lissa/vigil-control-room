import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getMe, login, logout, register, updateProfile } from "./api";
import { useAuthStore } from "./store";
import { LoginRequest, RegisterRequest, UpdateProfileRequest } from "./types";

export const useRegister = () => {
  const { setAuth } = useAuthStore();

  return useMutation({
    mutationFn: (body: RegisterRequest) => register(body),
    onSuccess: (data) => {
      setAuth(data.token, data.user);
    },
  });
};

export const useLogin = () => {
  const { setAuth } = useAuthStore();

  return useMutation({
    mutationFn: (body: LoginRequest) => login(body),
    onSuccess: (data) => {
      setAuth(data.token, data.user);
    },
  });
};

export const useLogout = () => {
  const { token, clearAuth } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => logout(token!),
    onSuccess: () => {
      clearAuth();
      queryClient.clear();
    },
  });
};

export const useMe = () => {
  const { token } = useAuthStore();

  return useQuery({
    queryKey: ["me"],
    queryFn: () => getMe(token!),
    enabled: !!token,
  });
};

export const useUpdateProfile = () => {
  const { token } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: UpdateProfileRequest) => updateProfile(body, token!),
    onSuccess: (data) => {
      queryClient.setQueryData(["me"], data);
    },
  });
};
