import { apiClient } from "@/shared/lib/api-client";
import {
  AuthResponse,
  LoginRequest,
  RegisterRequest,
  UpdateProfileRequest,
  UserResponse,
} from "./types";

export const register = (body: RegisterRequest): Promise<AuthResponse> =>
  apiClient.post<AuthResponse>("/register", body);

export const login = (body: LoginRequest): Promise<AuthResponse> =>
  apiClient.post<AuthResponse>("/login", body);

export const logout = (token: string): Promise<void> =>
  apiClient.post<void>("/logout", {}, token);

export const getMe = (token: string): Promise<UserResponse> =>
  apiClient.get<UserResponse>("/me", token);

export const updateProfile = (
  body: UpdateProfileRequest,
  token: string,
): Promise<UserResponse> => apiClient.patch<UserResponse>("/me", body, token);
