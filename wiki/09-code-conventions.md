# Code Conventions

## 왜 코드 컨벤션이 중요한가?

여러 사람이 한 프로젝트에서 일할 때, 같은 규칙을 따르면 코드를 읽기가 훨씬 쉬워진다. 마치 도로에서 모든 차가 같은 교통법규를 따라야 안전하고 원활하게 달릴 수 있는 것처럼, 코드도 일관된 규칙이 있어야 누구나 빠르게 이해하고 수정할 수 있다.

컨벤션이 없으면 사람마다 다른 스타일로 코드를 작성하게 되고, 나중에 "이 코드가 뭘 하는 거지?" 하는 시간이 기하급수적으로 늘어난다.

---

## 이 프로젝트에서 가장 중요한 규칙 TOP 5

| 순위 | 규칙 | 왜? |
|------|------|-----|
| 1 | Rust 모듈 파일은 `t_` 접두사 | 프로젝트 고유 파일을 한눈에 구분 |
| 2 | 에러 처리는 `Result<T, String>` | 프로젝트 전체에서 통일된 간결한 패턴 |
| 3 | Vue 컴포넌트는 `<script setup>` 전용 | Vue 3의 가장 간결한 방식, Options API 사용 금지 |
| 4 | Tailwind + daisyUI로만 스타일링 | scoped CSS 대신 유틸리티 클래스 통일 |
| 5 | Import 순서: std → 외부 → 내부 | 의존성 방향이 한눈에 보임 |

---

## Rust Backend

### File Naming
- 모든 모듈 파일은 `t_` 접두사 사용: `t_sqlite.rs`, `t_ai.rs`, `t_face.rs`
- "Tauri" 모듈을 의미하는 네임스페이스 컨벤션

> **`t_` 접두사가 왜 필요한가?**
> 프로젝트에는 수십 개의 `.rs` 파일이 있다. 외부 라이브러리 파일과 프로젝트 고유 모듈을 한눈에 구분하기 위해 `t_` 접두사를 붙인다. 마치 학교에서 각 반의 교과서에 "1반-수학", "1반-영어" 이렇게 반 번호를 붙이는 것과 비슷하다. `t_`는 "이 파일은 Tauri 앱의 핵심 모듈"이라는 이름표 역할을 한다. 파일 탐색기에서 `t_`로 시작하는 파일만 보면 프로젝트의 핵심 로직을 모두 파악할 수 있다.

### Naming Rules
| Type | Convention | Example |
|------|-----------|---------|
| Files | `t_` prefix + snake_case | `t_config.rs` |
| Functions | snake_case | `get_app_config`, `index_album` |
| Structs | PascalCase | `Album`, `AFile`, `FaceEngine` |
| Constants | SCREAMING_SNAKE_CASE | `MIN_CONFIDENCE`, `K_NEIGHBORS` |

### Error Handling
- 모든 함수 반환: `Result<T, String>` (typed error 없음, thiserror 미사용)
- 에러 전파: `.map_err(|e| e.to_string())?`
- 일부 함수는 에러 시 기본값 반환 (예: `get_image_orientation()` → `1`)

> **`Result<T, String>` 패턴이란?**
> 함수가 성공하면 `T`(원하는 값)를 주고, 실패하면 에러 메시지(`String`)를 준다. 편지봉투에 비유하면, 봉투를 열었을 때 **결과물**이 들어있거나 **"실패했습니다"라는 메모**가 들어있는 것이다.
>
> Rust에서는 `thiserror` 같은 라이브러리로 에러 타입을 세분화할 수도 있지만, 이 프로젝트는 단순하게 `String`으로 통일했다. 덕분에 모든 에러 처리 코드가 `.map_err(|e| e.to_string())?` 한 줄로 끝난다. 프로젝트 규모에서는 이 정도가 적절한 트레이드오프다.

```rust
// 이 프로젝트의 표준 에러 패턴
fn my_function() -> Result<T, String> {
    something().map_err(|e| e.to_string())?;
    Ok(result)
}
```

### Tauri Command 구조
```rust
// 동기 커맨드 (대부분)
#[tauri::command]
pub fn get_album(album_id: i64) -> Result<Album, String> {
    Album::get_by_id(album_id)
}

// 비동기 커맨드 (CPU-heavy 작업)
#[tauri::command]
pub async fn edit_image(params: EditParams) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || t_image::edit_image(params))
        .await
        .map_err(|e| e.to_string())?
}
```

### Serialization
- 거의 모든 struct에 `#[derive(Serialize, Deserialize)]`
- JSON 호환을 위해 `#[serde(rename_all = "camelCase")]` 사용
- 하위 호환: `#[serde(alias = "old_name")]`, `#[serde(default)]`

### 로깅
- `println!()` — 초기화 메시지 ("Loading AI Models...")
- `eprintln!()` — 에러/경고
- log 크레이트 미사용

### 주석 스타일
```rust
/**
 * Module description.
 * project: Lap
 * author:  julyx10
 */

/// get app config (libraries list and current library)
#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> { ... }
```

### Import 순서
```rust
use std::collections::HashMap;      // 1. std
use std::path::Path;
use tauri::State;                    // 2. External crates
use serde::{Deserialize, Serialize};
use crate::t_config;                 // 3. Internal modules
use crate::t_image;
```

### Formatting
- rustfmt 기본 설정 (커스텀 `.rustfmt.toml` 없음)
- 4-space indent
- Clippy 커스텀 설정 없음
- unsafe 코드: FFI 바인딩 외에는 없음

---

## Vue Frontend

### Component Style
- **`<script setup>` 전용** — Options API 사용하지 않음
- TypeScript 혼용: `.vue` 파일은 `lang="ts"`, 스토어는 `.js`

> **`<script setup>`이란?**
> Vue 3에서 컴포넌트를 만드는 가장 간결한 방법이다. 기존 Vue 2에서는 `export default { setup() { ... } }` 처럼 감싸야 했는데, `<script setup>`을 쓰면 별도의 `setup()` 함수 없이 바로 코드를 작성할 수 있다. 변수를 선언하면 자동으로 템플릿에서 사용 가능하고, `return`도 필요 없다. 요리에 비유하면, 기존 방식은 "재료를 꺼내고 → 도마에 올리고 → 칼을 꺼내고 → 자른다"였다면, `<script setup>`은 "재료와 도구가 이미 세팅된 요리 키트"와 같다.

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { config } from '@/common/config';

const props = defineProps({
  buttonSize: { type: String, default: 'medium' },
  icon: { type: Object as () => Component, required: false },
});
const emit = defineEmits(['click']);
</script>
```

### Component Naming
- 파일명: PascalCase (`AlbumList.vue`, `TButton.vue`)
- 일부 `T` 접두사 컴포넌트 존재 (`TButton`, `ToolTip`)

### Import 순서
```typescript
// 1. Vue
import { ref, computed, onMounted } from 'vue';
// 2. Tauri
import { listen } from '@tauri-apps/api/event';
// 3. Stores
import { useConfigStore } from '@/stores/configStore';
// 4. Components
import TButton from '@/components/TButton.vue';
// 5. Utils/icons
import { formatFileSize } from '@/common/utils';
```

### Styling
- **Tailwind + daisyUI 전용** — scoped CSS 거의 없음
- 동적 값만 inline style 사용
- 조건부 클래스: 배열 + 객체 바인딩

```vue
<div :class="[
  'btn btn-ghost btn-square',
  { 'btn-disabled': disabled },
  { 'btn-xs': buttonSize === 'small' }
]">
```

### daisyUI 주요 컴포넌트
`btn`, `badge`, `tabs`, `dropdown`, `modal`, `input`, `loading`, `skeleton`, `divider`, `breadcrumbs`, `table`, `menu`

### Color System
- Base: `base-100`, `base-200`, `base-300`, `base-content`
- Status: `primary`, `secondary`, `accent`, `success`, `warning`, `error`
- Opacity: `text-base-content/50` (50%)

### State Management 패턴
```javascript
// Pinia store (Options API 스타일)
export const useConfigStore = defineStore('configStore', {
  state: () => ({ ... }),
  actions: { ... },
  getters: { ... },
});
```

### 주석
- 최소한의 주석, 코드가 자체 설명적
- JSDoc 미사용
- 섹션 구분용 `//` 코멘트

### Formatting
- Prettier 기본 설정 (`.prettierrc` 없음)
- ESLint 미설정

---

## 처음 코드 작성할 때 체크리스트

새로운 코드를 작성하거나 기존 코드를 수정할 때, 이 체크리스트를 확인하자:

### Rust 코드
- [ ] 새 모듈 파일은 `t_` 접두사를 붙였는가?
- [ ] 함수 반환 타입이 `Result<T, String>`인가?
- [ ] 에러 전파에 `.map_err(|e| e.to_string())?`를 사용했는가?
- [ ] struct에 `#[derive(Serialize, Deserialize)]`를 추가했는가?
- [ ] JSON 호환을 위해 `#[serde(rename_all = "camelCase")]`를 붙였는가?
- [ ] Import 순서: std → 외부 크레이트 → 내부 모듈 순서인가?
- [ ] Tauri command에 `#[tauri::command]` 어트리뷰트를 붙였는가?
- [ ] CPU-heavy 작업은 `spawn_blocking`으로 감싸서 async 런타임을 블로킹하지 않는가?

### Vue 코드
- [ ] `<script setup lang="ts">`를 사용했는가? (Options API 금지)
- [ ] 컴포넌트 파일명이 PascalCase인가? (`AlbumList.vue`)
- [ ] 스타일링에 Tailwind + daisyUI 클래스를 사용했는가? (scoped CSS 지양)
- [ ] Import 순서: Vue → Tauri → Stores → Components → Utils 순서인가?
- [ ] `defineProps`와 `defineEmits`로 props/events를 정의했는가?
