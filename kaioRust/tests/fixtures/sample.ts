import { useState, useEffect } from 'react';
import axios from 'axios';
import { UserConfig, AuthToken } from './types';

export const API_URL = "https://api.example.com/v1";
const SECRET_KEY = "local-secret";

export interface UserProfile {
    id: string;
    username: string;
    email: string;
}

export enum UserRole {
    Admin = "ADMIN",
    Member = "MEMBER",
    Guest = "GUEST"
}

export type UserId = string | number;

export function formatUserName(profile: UserProfile): string {
    return `${profile.username} <${profile.email}>`;
}

function internalHelper(token: AuthToken): boolean {
    return token.expiresAt > Date.now();
}

export const fetchUserById = async (id: UserId): Promise<UserProfile> => {
    const res = await axios.get(`${API_URL}/users/${id}`);
    return res.data;
};

export class UserService {
    private client: any;

    constructor(client: any) {
        this.client = client;
    }

    public async getUser(id: string): Promise<UserProfile> {
        return this.client.get(id);
    }

    protected validateId(id: string): boolean {
        return id.length > 0;
    }

    private logAccess(id: string): void {
        console.log(`Access user ${id}`);
    }
}

export { internalHelper };
