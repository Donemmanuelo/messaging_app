import React, { useEffect, useState } from 'react';
import { View, Text, Button, FlatList, TouchableOpacity, Alert } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';

export default function ChatListScreen({ navigation }) {
  const [chats, setChats] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    AsyncStorage.getItem('token').then(token => {
      if (!token) {
        navigation.replace('Login');
        return;
      }
      fetch('http://localhost:3001/api/chats', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
        .then(res => {
          if (res.status === 401) {
            AsyncStorage.removeItem('token');
            navigation.replace('Login');
            return [];
          }
          return res.json();
        })
        .then(setChats)
        .finally(() => setLoading(false));
    });
  }, []);

  const handleLogout = async () => {
    await AsyncStorage.removeItem('token');
    navigation.replace('Login');
  };

  if (loading) return <Text>Loading chats...</Text>;

  return (
    <View style={{ flex: 1, padding: 16 }}>
      <Text style={{ fontSize: 24, marginBottom: 16 }}>Chats</Text>
      <Button title="Logout" onPress={handleLogout} />
      <FlatList
        data={chats}
        keyExtractor={item => item.id}
        renderItem={({ item }) => (
          <TouchableOpacity onPress={() => navigation.navigate('ChatRoom', { chatId: item.id })}>
            <Text style={{ fontSize: 18, marginBottom: 8 }}>{item.name || 'Chat'}</Text>
          </TouchableOpacity>
        )}
      />
    </View>
  );
} 