import { useEffect } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getMe, login, logout, register, updateProfile } from "./api";
import { getToken, setToken, clearToken } from "./token";
import { LoginRequest, RegisterRequest, UpdateProfileRequest } from "./types";

export const useRegister = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: RegisterRequest) => register(body),
    onSuccess: (data) => {
      setToken(data.token);
      queryClient.setQueryData(["me"], data.user);
    },
  });
};

export const useLogin = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: LoginRequest) => login(body),
    onSuccess: (data) => {
      setToken(data.token);
      queryClient.setQueryData(["me"], data.user);
    },
  });
};

export const useHandleOAuthCallback = () => {
  const router = useRouter();
  const searchParams = useSearchParams();

  useEffect(() => {
    const token = searchParams.get("token");
    if (token) {
      setToken(token);
      router.replace("/incidents");
    } else {
      router.replace("/login");
    }
  }, [searchParams, router]);
};

export const useLogout = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => logout(getToken()!),
    onSuccess: () => {
      clearToken();
      queryClient.clear();
    },
  });
};

export const useMe = () =>
  useQuery({
    queryKey: ["me"],
    queryFn: () => {
      const token = getToken();
      if (!token) return Promise.reject(new Error("No token"));
      return getMe(token);
    },
  });

export const useUpdateProfile = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: UpdateProfileRequest) =>
      updateProfile(body, getToken()!),
    onSuccess: (data) => {
      queryClient.setQueryData(["me"], data);
    },
  });
};
