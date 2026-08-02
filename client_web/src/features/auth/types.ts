export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface UpdateProfileRequest {
  username?: string;
  language?: string;
  new_password?: string;
  old_password?: string;
}

export interface UserResponse {
  id: string;
  username: string;
  email: string;
  language: string;
}

export interface LoginResponse {
  token: string;
  user: UserResponse;
}
