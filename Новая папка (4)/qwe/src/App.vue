<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface PasswordEntry {
  name: string;
  username: string;
  password: string;
  url?: string;
  notes?: string;
}

const passwords = ref<PasswordEntry[]>([]);
const showAddForm = ref(false);
const searchQuery = ref("");
const selectedPassword = ref<PasswordEntry | null>(null);
const showPassword = ref(false);
const errorMessage = ref("");

// Form data
const newPassword = ref<PasswordEntry>({
  name: "",
  username: "",
  password: "",
  url: "",
  notes: ""
});

// Password generator
const passwordLength = ref(16);

onMounted(async () => {
  await loadPasswords();
});

async function loadPasswords() {
  try {
    passwords.value = await invoke<PasswordEntry[]>("get_passwords");
  } catch (error) {
    errorMessage.value = `Ошибка при загрузке паролей: ${error}`;
  }
}

async function addPassword() {
  try {
    await invoke("add_password", {
      name: newPassword.value.name,
      username: newPassword.value.username,
      password: newPassword.value.password,
      url: newPassword.value.url || null,
      notes: newPassword.value.notes || null
    });
    
    // Reset form
    newPassword.value = {
      name: "",
      username: "",
      password: "",
      url: "",
      notes: ""
    };
    
    showAddForm.value = false;
    await loadPasswords();
  } catch (error) {
    errorMessage.value = `Ошибка при добавлении пароля: ${error}`;
  }
}

async function deletePassword(name: string) {
  if (confirm(`Вы уверены, что хотите удалить запись "${name}"?`)) {
    try {
      await invoke("delete_password", { name });
      await loadPasswords();
      if (selectedPassword.value?.name === name) {
        selectedPassword.value = null;
      }
    } catch (error) {
      errorMessage.value = `Ошибка при удалении пароля: ${error}`;
    }
  }
}

async function generatePassword() {
  try {
    newPassword.value.password = await invoke<string>("generate_password", { length: passwordLength.value });
  } catch (error) {
    errorMessage.value = `Ошибка при генерации пароля: ${error}`;
  }
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text)
    .then(() => {
      // Можно добавить уведомление об успешном копировании
    })
    .catch(err => {
      errorMessage.value = `Ошибка при копировании: ${err}`;
    });
}

function getFilteredPasswords() {
  if (!searchQuery.value) return passwords.value;
  
  const query = searchQuery.value.toLowerCase();
  return passwords.value.filter(p => 
    p.name.toLowerCase().includes(query) || 
    p.username.toLowerCase().includes(query) ||
    (p.url && p.url.toLowerCase().includes(query))
  );
}
</script>

<template>
  <main class="container">
    <h1>Менеджер паролей</h1>
    
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
      <button @click="errorMessage = ''">✕</button>
    </div>
    
    <div class="top-controls">
      <button @click="showAddForm = true" class="add-btn">Добавить пароль</button>
      <input 
        type="search" 
        v-model="searchQuery" 
        placeholder="Поиск..." 
        class="search-input"
      />
    </div>
    
    <div class="main-content">
      <!-- Список паролей -->
      <div class="password-list">
        <div v-if="getFilteredPasswords().length === 0" class="empty-state">
          {{ searchQuery ? 'Нет результатов по запросу' : 'Нет сохраненных паролей' }}
        </div>
        
        <div 
          v-for="password in getFilteredPasswords()" 
          :key="password.name"
          class="password-item"
          :class="{ 'selected': selectedPassword?.name === password.name }"
          @click="selectedPassword = password"
        >
          <div class="item-name">{{ password.name }}</div>
          <div class="item-username">{{ password.username }}</div>
          <button 
            @click.stop="deletePassword(password.name)" 
            class="delete-btn"
            title="Удалить"
          >✕</button>
        </div>
      </div>
      
      <!-- Детали пароля -->
      <div class="password-details" v-if="selectedPassword">
        <h2>{{ selectedPassword.name }}</h2>
        
        <div class="detail-item">
          <label>Логин:</label>
          <div class="detail-value">
            {{ selectedPassword.username }}
            <button 
              @click="copyToClipboard(selectedPassword.username)" 
              class="copy-btn"
              title="Копировать"
            >📋</button>
          </div>
        </div>
        
        <div class="detail-item">
          <label>Пароль:</label>
          <div class="detail-value">
            <span v-if="showPassword">{{ selectedPassword.password }}</span>
            <span v-else>••••••••</span>
            <button 
              @click="showPassword = !showPassword" 
              class="view-btn"
              :title="showPassword ? 'Скрыть' : 'Показать'"
            >👁️</button>
            <button 
              @click="copyToClipboard(selectedPassword.password)" 
              class="copy-btn"
              title="Копировать"
            >📋</button>
          </div>
        </div>
        
        <div class="detail-item" v-if="selectedPassword.url">
          <label>URL:</label>
          <div class="detail-value">
            <a :href="selectedPassword.url" target="_blank">{{ selectedPassword.url }}</a>
          </div>
        </div>
        
        <div class="detail-item" v-if="selectedPassword.notes">
          <label>Заметки:</label>
          <div class="detail-value notes">{{ selectedPassword.notes }}</div>
        </div>
      </div>
    </div>
    
    <!-- Модальное окно добавления пароля -->
    <div class="modal" v-if="showAddForm">
      <div class="modal-content">
        <h2>Добавить новый пароль</h2>
        
        <form @submit.prevent="addPassword">
          <div class="form-group">
            <label for="name">Название:</label>
            <input type="text" id="name" v-model="newPassword.name" required />
          </div>
          
          <div class="form-group">
            <label for="username">Логин:</label>
            <input type="text" id="username" v-model="newPassword.username" required />
          </div>
          
          <div class="form-group">
            <label for="password">Пароль:</label>
            <div class="password-input-group">
              <input type="text" id="password" v-model="newPassword.password" required />
              <button type="button" @click="generatePassword" class="generate-btn">Сгенерировать</button>
            </div>
            <div class="password-generator">
              <label for="length">Длина пароля: {{ passwordLength }}</label>
              <input 
                type="range" 
                id="length" 
                v-model="passwordLength" 
                min="8" 
                max="32" 
                step="1"
              />
            </div>
          </div>
          
          <div class="form-group">
            <label for="url">URL (необязательно):</label>
            <input type="url" id="url" v-model="newPassword.url" />
          </div>
          
          <div class="form-group">
            <label for="notes">Заметки (необязательно):</label>
            <textarea id="notes" v-model="newPassword.notes"></textarea>
          </div>
          
          <div class="form-actions">
            <button type="button" @click="showAddForm = false" class="cancel-btn">Отмена</button>
            <button type="submit" class="save-btn">Сохранить</button>
          </div>
        </form>
      </div>
    </div>
  </main>
</template>

<style>
:root {
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;

  color: #213547;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  display: flex;
  min-width: 320px;
  min-height: 100vh;
}

.container {
  max-width: 1000px;
  margin: 0 auto;
  padding: 2rem;
  text-align: center;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

h1 {
  font-size: 2em;
  margin-bottom: 1.5rem;
  color: #213547;
}

.error-message {
  background-color: #ffeded;
  color: #b00020;
  padding: 0.75rem;
  border-radius: 4px;
  margin-bottom: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.error-message button {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.2rem;
  color: #b00020;
}

.top-controls {
  display: flex;
  justify-content: space-between;
  margin-bottom: 1rem;
}

button {
  border-radius: 4px;
  border: 1px solid transparent;
  padding: 0.5em 1em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  background-color: #f9f9f9;
  cursor: pointer;
  transition: border-color 0.25s;
}

button:hover {
  border-color: #646cff;
}

input, textarea {
  border-radius: 4px;
  border: 1px solid #ddd;
  padding: 0.5em 1em;
  font-size: 1em;
  font-family: inherit;
  width: 100%;
}

.search-input {
  padding-left: 2rem;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%23aaa' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Ccircle cx='11' cy='11' r='8'/%3E%3Cline x1='21' y1='21' x2='16.65' y2='16.65'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: 0.5rem center;
  width: 250px;
}

.add-btn {
  background-color: #4f46e5;
  color: white;
}

.main-content {
  display: flex;
  gap: 2rem;
  height: 100%;
  overflow: hidden;
  flex: 1;
}

.password-list {
  flex: 1;
  border: 1px solid #ddd;
  border-radius: 4px;
  overflow-y: auto;
}

.password-details {
  flex: 2;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 1rem;
  text-align: left;
  overflow-y: auto;
}

.empty-state {
  padding: 2rem;
  color: #888;
}

.password-item {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  display: flex;
  position: relative;
  text-align: left;
}

.password-item:hover {
  background-color: #f9f9f9;
}

.password-item.selected {
  background-color: #eef2ff;
}

.item-name {
  font-weight: 500;
  margin-bottom: 0.25rem;
}

.item-username {
  font-size: 0.85em;
  color: #666;
}

.delete-btn {
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: #888;
  display: none;
}

.password-item:hover .delete-btn {
  display: block;
}

.detail-item {
  margin-bottom: 1rem;
}

.detail-item label {
  font-weight: 500;
  display: block;
  margin-bottom: 0.25rem;
  color: #666;
}

.detail-value {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.copy-btn, .view-btn {
  background: none;
  border: none;
  padding: 0.25rem;
  font-size: 1rem;
}

.notes {
  white-space: pre-wrap;
}

.modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background-color: white;
  padding: 2rem;
  border-radius: 8px;
  width: 100%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 1rem;
  text-align: left;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.password-input-group {
  display: flex;
  gap: 0.5rem;
}

.password-generator {
  margin-top: 0.5rem;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  margin-top: 1.5rem;
}

.save-btn {
  background-color: #4f46e5;
  color: white;
}

.cancel-btn {
  background-color: #f3f4f6;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: rgba(255, 255, 255, 0.87);
    background-color: #242424;
  }
  
  h1, h2 {
    color: rgba(255, 255, 255, 0.87);
  }
  
  .password-list, .password-details {
    border-color: #333;
    background-color: #1a1a1a;
  }
  
  .password-item {
    border-bottom-color: #333;
  }
  
  .password-item:hover {
    background-color: #2a2a2a;
  }
  
  .password-item.selected {
    background-color: #2c3e50;
  }
  
  input, textarea {
    background-color: #1a1a1a;
    color: rgba(255, 255, 255, 0.87);
    border-color: #333;
  }
  
  button {
    background-color: #2a2a2a;
    color: rgba(255, 255, 255, 0.87);
  }
  
  .add-btn, .save-btn {
    background-color: #4f46e5;
  }
  
  .cancel-btn {
    background-color: #3a3a3a;
  }
  
  .modal-content {
    background-color: #242424;
  }
}
</style>