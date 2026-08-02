import { apiClient } from "@/shared/lib/api-client";
import {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  UpdateProfileRequest,
  UserResponse,
} from "./types";

export const register = (body: RegisterRequest): Promise<UserResponse> =>
  apiClient.post<UserResponse>("/register", body);

export const login = (body: LoginRequest): Promise<LoginResponse> =>
  apiClient.post<LoginResponse>("/login", body);

export const logout = (token: string): Promise<void> =>
  apiClient.post<void>("/logout", {}, token);

export const getMe = (token: string): Promise<UserResponse> =>
  apiClient.get<UserResponse>("/me", token);

export const updateProfile = (
  body: UpdateProfileRequest,
  token: string,
): Promise<UserResponse> => apiClient.patch<UserResponse>("/me", body, token);
