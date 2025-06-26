import React, { useEffect, useState } from 'react';
import { View, Text, FlatList, TextInput, Button, Image } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';

export default function ChatRoomScreen({ route }) {
  const { chatId } = route.params;
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState('');
  const [token, setToken] = useState('');

  useEffect(() => {
    AsyncStorage.getItem('token').then(t => {
      if (!t) return;
      setToken(t);
      fetchMessages(t);
    });
  }, [chatId]);

  const fetchMessages = (token: string) => {
    fetch(`http://localhost:3001/api/messages/${chatId}`, {
      headers: { 'Authorization': `Bearer ${token}` }
    })
      .then(res => res.json())
      .then(setMessages);
  };

  const sendMessage = async () => {
    if (!input.trim() || !token) return;
    await fetch(`http://localhost:3001/api/messages/${chatId}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ content: input }),
    });
    setInput('');
    fetchMessages(token);
  };

  const renderMessage = ({ item }) => (
    <View style={{ flexDirection: 'row', alignItems: 'flex-end', marginBottom: 8 }}>
      <Image source={{ uri: item.sender_avatar || 'https://placehold.co/40x40' }} style={{ width: 32, height: 32, borderRadius: 16, marginRight: 8 }} />
      <View style={{ backgroundColor: '#e5e7eb', borderRadius: 12, padding: 8, maxWidth: '80%' }}>
        <Text style={{ fontWeight: 'bold' }}>{item.sender_name || 'User'}</Text>
        <Text>{item.content}</Text>
        <Text style={{ fontSize: 10, color: '#888', alignSelf: 'flex-end' }}>{new Date(item.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</Text>
      </View>
    </View>
  );

  return (
    <View style={{ flex: 1, padding: 16 }}>
      <FlatList
        data={messages}
        keyExtractor={item => item.id}
        renderItem={renderMessage}
      />
      <View style={{ flexDirection: 'row', alignItems: 'center' }}>
        <TextInput
          value={input}
          onChangeText={setInput}
          style={{ flex: 1, borderWidth: 1, padding: 8, marginRight: 8 }}
          placeholder="Type a message"
        />
        <Button title="Send" onPress={sendMessage} />
      </View>
    </View>
  );
} 