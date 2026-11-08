import React, { useState } from 'react';
import { Button } from './components/button';

export interface CardProps {
    title: string;
    description?: string;
    onAction?: () => void;
}

export const ActionCard: React.FC<CardProps> = ({ title, description, onAction }) => {
    const [count, setCount] = useState(0);

    return (
        <div className="action-card">
            <h2>{title}</h2>
            {description && <p>{description}</p>}
            <button onClick={() => setCount(count + 1)}>Clicks: {count}</button>
            <Button onClick={onAction}>Execute</Button>
        </div>
    );
};

export default ActionCard;
