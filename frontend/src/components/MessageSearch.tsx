import React, { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';

interface SearchResult {
    id: string;
    content: string;
    created_at: string;
    sender: {
        id: string;
        username: string;
    };
    chat: {
        id: string;
        name: string;
        is_group: boolean;
    };
}

interface MessageSearchProps {
    chatId?: string;
    onResultClick?: (result: SearchResult) => void;
}

export const MessageSearch: React.FC<MessageSearchProps> = ({
    chatId,
    onResultClick,
}) => {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState<SearchResult[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [showResults, setShowResults] = useState(false);
    const router = useRouter();

    useEffect(() => {
        const searchTimeout = setTimeout(async () => {
            if (query.trim().length < 2) {
                setResults([]);
                return;
            }

            setIsLoading(true);
            try {
                const params = new URLSearchParams({
                    query: query.trim(),
                    ...(chatId && { chat_id: chatId }),
                });

                const response = await fetch(`/api/messages/search?${params}`);
                if (!response.ok) {
                    throw new Error('Search failed');
                }

                const data = await response.json();
                setResults(data);
            } catch (error) {
                console.error('Search error:', error);
            } finally {
                setIsLoading(false);
            }
        }, 300);

        return () => clearTimeout(searchTimeout);
    }, [query, chatId]);

    const handleResultClick = (result: SearchResult) => {
        if (onResultClick) {
            onResultClick(result);
        } else {
            // Navigate to the chat and scroll to the message
            router.push(
                `/chat/${result.chat.is_group ? 'group' : 'direct'}/${
                    result.chat.id
                }?message=${result.id}`
            );
        }
        setShowResults(false);
    };

    return (
        <div className="relative">
            <div className="relative">
                <input
                    type="text"
                    value={query}
                    onChange={(e) => {
                        setQuery(e.target.value);
                        setShowResults(true);
                    }}
                    onFocus={() => setShowResults(true)}
                    placeholder="Search messages..."
                    className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                {isLoading && (
                    <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
                        <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-500" />
                    </div>
                )}
            </div>

            {showResults && results.length > 0 && (
                <div className="absolute z-10 w-full mt-1 bg-white rounded-lg shadow-lg border max-h-96 overflow-y-auto">
                    {results.map((result) => (
                        <button
                            key={result.id}
                            onClick={() => handleResultClick(result)}
                            className="w-full px-4 py-3 text-left hover:bg-gray-50 border-b last:border-b-0"
                        >
                            <div className="flex items-start gap-3">
                                <div className="flex-1 min-w-0">
                                    <div className="flex items-center gap-2">
                                        <span className="font-medium">
                                            {result.sender.username}
                                        </span>
                                        <span className="text-sm text-gray-500">
                                            in{' '}
                                            {result.chat.is_group
                                                ? result.chat.name
                                                : 'Direct Message'}
                                        </span>
                                    </div>
                                    <p className="mt-1 text-sm text-gray-600 truncate">
                                        {result.content}
                                    </p>
                                    <p className="mt-1 text-xs text-gray-400">
                                        {new Date(
                                            result.created_at
                                        ).toLocaleString()}
                                    </p>
                                </div>
                            </div>
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}; 