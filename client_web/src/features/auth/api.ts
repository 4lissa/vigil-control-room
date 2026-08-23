import { API_BASE_URL, apiClient } from "@/shared/lib/api-client";
import {
  AuthResponse,
  ConnectServiceRequest,
  ConnectedServiceStatusResponse,
  LoginRequest,
  RegisterRequest,
  UpdateProfileRequest,
  UserResponse,
} from "./types";

export const register = (body: RegisterRequest): Promise<AuthResponse> =>
  apiClient.post<AuthResponse>("/register", body);

export const login = (body: LoginRequest): Promise<AuthResponse> =>
  apiClient.post<AuthResponse>("/login", body);

export const githubSignInUrl = `${API_BASE_URL}/github`;

export const logout = (token: string): Promise<void> =>
  apiClient.post<void>("/logout", {}, token);

export const getMe = (token: string): Promise<UserResponse> =>
  apiClient.get<UserResponse>("/me", token);

export const updateProfile = (
  body: UpdateProfileRequest,
  token: string,
): Promise<UserResponse> => apiClient.patch<UserResponse>("/me", body, token);

export const connectService = (
  service: string,
  body: ConnectServiceRequest,
  token: string,
): Promise<void> =>
  apiClient.post<void>(`/connected-services/${service}`, body, token);

export const getConnectedServiceStatus = (
  service: string,
  token: string,
): Promise<ConnectedServiceStatusResponse> =>
  apiClient.get<ConnectedServiceStatusResponse>(
    `/connected-services/${service}`,
    token,
  );

export const disconnectService = (
  service: string,
  token: string,
): Promise<void> =>
  apiClient.delete<void>(`/connected-services/${service}`, token);
