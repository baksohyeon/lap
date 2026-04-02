# Design Decisions — 이 개발자는 어떻게 이런 프로젝트를 만들었을까

614개 커밋을 시간순으로 읽으며 추론한, julyx10의 설계 여정과 기술적 판단들.

---

## 한 사람의 여정

이 프로젝트는 처음부터 완성된 아키텍처가 있었던 게 아니다. 커밋 히스토리를 보면 **삽질하고, 되돌리고, 다시 시도하는** 과정이 그대로 보인다. 그게 오히려 대단한 점이다.

---

## Phase 1: "일단 돌아가게 만들자" (2024.08 ~ 09)

첫 50개 커밋은 거의 전부 `bugfix`, `bug fix`, `bugfix of panic`이다.

```
69c124f init
cb8572a add vite and vue frontend
72c9d37 fetch folder tree and display in the pane
97b8f38 add thumb fn
2c95ebb bugfix of panic
9411c3f bug fix
8253d4d bug fix
7faf8dc bug fix
5293dac bugfix
4eb7e4c bug fix
32ea4fe bugfix
2f8d533 bug fix
```

**추론**: 처음에는 Rust + Tauri + Vue 조합 자체가 낯설었을 것이다. SQLite에서 사진 메타데이터 읽기, 썸네일 생성, 이미지 뷰어 구현 — 각각은 간단하지만 합치면 수많은 엣지 케이스가 터진다. EXIF 파싱에서 panic, 이미지 로드 실패, 윈도우 이벤트 처리... 이 시기에 `t_` 접두사 컨벤션이 이미 잡혀있는 걸 보면, 백엔드 모듈 설계는 초기부터 의식적이었다.

**이 시기의 핵심 결정**:

- Tauri 1.0 선택 (나중에 2.0으로 마이그레이션)
- SQLite에 모든 메타데이터 저장 (파일 기반 DB = 오프라인 퍼스트의 기반)
- Vue + Vite 프론트엔드 (React 대신 — 아마 SFC의 단순함 선호)

### 왜 Tauri + Rust였을까?

데스크탑 프레임워크 선택지는 많았다. 하지만 이 앱의 요구사항을 놓고 보면 Tauri + Rust가 거의 유일한 선택지였을 것이다.

| | Electron | Tauri + Rust | Flutter | Native (Swift/.NET) |
|---|---|---|---|---|
| C/C++ FFI (LibRaw, FFmpeg, ONNX) | node-gyp addon (빌드 지옥) | **Rust FFI zero-cost** | Dart FFI (생태계 작음) | 가능하지만 플랫폼 한정 |
| 멀티플랫폼 | O | O | O | Swift=macOS, .NET=Windows |
| 앱 크기 | 100MB+ (Chromium 번들) | 10~20MB | 30~50MB | 가벼움 |
| 메모리 (수만 장 처리) | V8 GC 한계 | 수동 제어 | Dart GC 한계 | 좋음 |
| AI 추론 로컬 실행 | node-onnxruntime (불안정) | `ort` crate (안정적) | 제한적 | CoreML (Apple only) |

**결정적 이유는 C/C++ FFI다.** 이 앱은 C/C++ 라이브러리를 3개나 직접 연동한다:

1. **LibRaw** (C++) — `build.rs`에서 소스 컴파일 + `libraw_shim.cpp` 래퍼
2. **FFmpeg** (C) — `ffmpeg-next` crate로 정적/동적 링킹
3. **ONNX Runtime** (C++) — `ort` crate로 AI 모델 추론

Rust에서 C/C++ 호출은 zero-overhead이고, `cc` crate로 빌드 시스템을 `Cargo.toml` 하나에 통합할 수 있다. Electron에서 같은 걸 하려면 node-gyp, N-API addon, 플랫폼별 prebuild 바이너리 관리가 필요한데 — Windows CI 커밋(`again and again`)을 보면 Rust에서도 힘든데, Node.js에서 했으면 포기했을 것이다.

Flutter나 Native(Swift)는 멀티플랫폼이 안 되거나, 웹뷰 기반 UI의 유연성이 없다. 사진 관리 앱의 복잡한 UI(justified layout, virtual scroll, 34개 테마, drag & drop)를 Swift에서 구현하는 건 Vue + Tailwind보다 몇 배 느렸을 것이다.

### "Python으로 패키징하면 안 되나?"

된다. PyInstaller, cx_Freeze로 Python 앱을 단일 실행 파일로 만들 수 있고, 실제로 그렇게 하는 프로젝트도 많다. "Rust가 항상 낫다"가 아니라 **이 앱의 요구사항에 Rust가 가장 맞았다**는 거다.

| | Python (PyInstaller) | Rust (Tauri) |
|---|---|---|
| 패키징 크기 | 300~500MB (Python runtime + PyTorch + FFmpeg) | 30~50MB |
| 앱 시작 시간 | 5~15초 (인터프리터 로드) | 1~2초 |
| 메모리 (사진 1만 장) | GC 기반, 예측 어려움 | 수동 제어, 일정함 |
| AI 추론 속도 | PyTorch가 더 유연 | ONNX Runtime과 거의 동일 |
| 크로스 플랫폼 빌드 | 각 OS에서 별도 패키징 | `cargo tauri build` 한 번에 |
| C/C++ 연동 | ctypes/cffi (가능하지만 오버헤드) | FFI zero-cost |
| UI 선택지 | Tkinter(구림), PyQt(라이선스), Electron+Python(복잡) | 웹뷰 (Vue + Tailwind) |

**Python이 더 나은 경우도 있다:**
- AI 모델 실험/프로토타이핑이 잦은 경우
- 이미 PyTorch 파이프라인이 있는 경우
- UI가 단순한 경우 (CLI 도구, 대시보드 등)

**Lap에서 Rust가 맞았던 이유:**
- 사진 수만 장을 빠르게 스크롤 → GC 없는 Rust가 유리
- "앱 설치 = 클릭 한 번"이어야 함 → 300MB Python 번들 vs 30MB Tauri
- C++ 라이브러리 3개를 직접 빌드해야 함 → Rust `build.rs`가 압도적으로 편함
- 34개 테마 + justified layout + virtual scroll → 웹 UI가 네이티브보다 빠르게 구현 가능

만약 AI 연구 도구였으면 Python이 맞았을 거고, 모바일 앱이었으면 Flutter가 맞았을 것이다. 기술 선택은 "뭐가 좋냐"가 아니라 **"이 제품이 뭘 해야 하냐"**에 따라 달라진다.

**결론**: "C/C++ 라이브러리를 3개 연동하면서 + 3개 OS를 지원하면서 + 복잡한 UI를 빠르게 만들 수 있는" 조합은 사실상 Tauri + Rust뿐이었다.

---

## Phase 2: "사용할 수 있는 앱으로" (2024.10 ~ 12)

```
4a32210 tauri 1.0 bak          ← Tauri 2.0 마이그레이션 직전 백업
67b103e Upgrade Tauri from 1.0 to 2.0
9c06c42 import pinia and pinia-plugin-persistedstate
265556d mac support
819cdd2 merge win and mac to main
```

**Tauri 1→2 마이그레이션**: `tauri 1.0 bak`이라는 커밋이 있다. 마이그레이션 전에 백업을 만들었다는 건 **되돌릴 수 있게 준비했다**는 뜻이다. 실제로 Tauri 2는 IPC API, 플러그인 시스템, 보안 모델이 완전히 바뀌었기 때문에 상당한 작업이었을 것이다.

**왜 이 시점에 마이그레이션했나?** Tauri 2의 capability 기반 보안 모델과 플러그인 생태계(`window-state`, `updater`, `dialog`)가 필요했을 것이다. 특히 `window-state` 플러그인은 이전에는 수동으로 구현해야 했던 창 위치/크기 복원을 한 줄로 해결한다.

**이 시기의 삽질들**:

```
b67a00c bugfix of exif_data: 1. trim extra punctuation and space
        2. panic of split_at(10) if string len less 10
```

EXIF 데이터의 날짜 문자열이 10자 미만일 때 `split_at(10)`이 panic하는 버그. 실제 카메라들이 다양한 형식의 EXIF를 쓰기 때문에 이런 엣지 케이스가 끝없이 나온다.

```
c11190ef bye 2024: drag albums to change display order impl.
```

2024년 마지막 커밋. 개발자의 성격이 보이는 커밋 메시지.

                                                                                   

⏺ "bye 2024" — 2024년 마지막 날에도 코딩하고 있었다는 뜻이야. 연말에 쉬지 않고 기능 구현하면서 "잘 가 2024"라고 인사한 거지.                                          

보이는 성격: 이 사람은 이 프로젝트를 진짜 좋아한다. 12월 31일에도 album drag order 기능을 넣고 있을 정도로. 억지로 하는 게 아니라 즐기면서 만드는 사람이라는 느낌.   

---

## Phase 3: "기능을 쌓아올리기" (2025.01 ~ 중반)

```
61f51eb intro daisyui to toggle theme
a50c6d0 applied daisyui (big change)    ← 본인도 "big change"라고 인정
```

**daisyUI 도입**: 이 커밋 메시지에 "(big change)"라고 직접 적었다. 전체 UI를 daisyUI 테마 시스템으로 전환한 대규모 리팩토링이었을 것이다. 34개 테마를 지원하려면 모든 하드코딩된 색상을 CSS 변수로 바꿔야 한다.

### 쓰레기통 기능의 진화 — 실패에서 배우는 과정

```
98f8149 impl trash(move to trash, restore from trash)
1856e82 impl trash folder
02da4ea impl trash album
...
e8b9eeb removed trash pane and related functions and used system default
        trash crate when deleting files or folders
536d8da impl delete files or folders using trash crate
```

**커밋을 읽어보면**: 처음에 직접 쓰레기통 UI를 구현했다. 폴더/앨범/파일 각각의 trash 기능을 만들고, 관련 버그를 여러 개 수정했다. 그런데 결국 **전부 삭제하고** OS 기본 trash crate로 교체했다. "내가 만든 것보다 OS 기본 기능이 낫다"는 판단을 한 것이다. 이건 경험 있는 개발자의 판단이다.

### 비디오 플레이어 선택의 여정

```
aaf23e3 add xgplayer to play video files.
fa8fb06 bugfix of xgplayer
2595950 replaced xgplayer with video.js
```

xgplayer를 먼저 시도했다가 video.js로 교체. 왜? xgplayer는 중국 회사(ByteDance) 제품으로 중국어 문서가 많지만, video.js가 더 성숙하고 테마 커스터마이징이 쉬움. **두 줄의 커밋에 기술 평가 → 도입 → 포기 → 대안 채택의 과정이 압축되어 있다.**

### 이미지 에디터의 진화

```
bcde24e impl: image editor using Cropper.js
afc8888 impl imageEditor.vue: remove cropper.js
ef5cfda trival updates on Image.vue
f12533e impl crop-box in ImageEdit.vue
```

Cropper.js 라이브러리를 먼저 도입했다가, 직접 crop 기능을 구현하기로 결정했다. 라이브러리가 Tauri WebView에서 제대로 작동하지 않았거나, 커스터마이징이 부족했을 것이다. **결국 직접 구현이 더 나은 경우**를 판단한 사례.

---

## Phase 4: "AI를 넣자" (2025 후반)

```
95bd1c2 Implement AI-powered hybrid image search with vector similarity
```

**왜 ONNX Runtime인가?**

이 개발자는 처음부터 **오프라인 퍼스트**를 설계 원칙으로 잡았다. AI를 넣되 클라우드 API를 쓰지 않겠다는 결정은 쉽지 않다. 선택지는:


| 방식                  | 장점               | 단점                   |
| ------------------- | ---------------- | -------------------- |
| OpenAI API 호출       | 구현 간단            | 인터넷 필요, 비용, 프라이버시 침해 |
| Python 서버 (PyTorch) | 모델 선택 자유         | 유저가 Python 설치해야 함    |
| ONNX Runtime (Rust) | 오프라인, 빠름, 설치 불필요 | 학습된 모델을 ONNX로 변환해야 함 |


ONNX를 선택했다는 건 **"유저에게 Python을 설치하라고 할 수 없다"**는 제품적 판단이다. 데스크탑 앱에서 사용자 경험을 최우선으로 놓은 결정.

**CLIP 모델 선택**: OpenAI의 CLIP은 텍스트-이미지 검색에서 사실상 표준이다. ONNX로 변환된 모델을 `resources/models/`에 번들하면 앱 설치만으로 AI 검색이 된다.

**InsightFace (RetinaFace + MobileFaceNet) 선택**: 얼굴 인식에서 가장 균형 잡힌 정확도/속도 트레이드오프. MobileFaceNet은 이름 그대로 모바일에서도 돌아갈 만큼 가볍다.

---

## Phase 5: "Custom Protocol 도입" (2026.03)

```
1399a1e feat: Implement a custom `thumb://` URI scheme to serve raw thumbnail
        images directly from the database, replacing base64 data URLs.
9361d02 Use preview protocol for raw and tiff images
```

**이전에는 어떻게 했나?** base64 data URL이었다. 즉:

```
<!-- 이전: base64로 인라인 -->
<img src="data:image/jpeg;base64,/9j/4AAQ..." />

<!-- 이후: 커스텀 프로토콜 -->
<img src="thumb://localhost/1/4523" />
```

**왜 바꿨나?**

1. base64는 원본보다 33% 크다 (바이너리 → 텍스트 변환 오버헤드)
2. 수천 개의 data URL이 DOM에 있으면 메모리 폭발
3. 브라우저 캐시를 활용할 수 없다 (data URL은 캐시 불가)
4. `Cache-Control: max-age=31536000, immutable` 헤더로 1년 캐시 가능

이 결정은 **v0.1.12에서야 나왔다**. 프로젝트 후반부. 즉, 처음부터 이걸 알고 있었던 게 아니라 **성능 문제를 겪은 후에** 해결책을 찾은 것이다. 대규모 사진 라이브러리에서 스크롤이 버벅였을 것이고, 프로파일링 결과 base64 인코딩이 병목이라는 걸 발견했을 것이다.

---

## Phase 6: "RAW 이미지 지원" (2026.03 후반)

```
c6c37cb feat: Add RAW files support.
8854952 feat: Implement RAW image support.
27e081c feat: replace rsraw dependency with libraw
```

**rsraw → LibRaw 소스 컴파일로 전환**: 처음에는 `rsraw`라는 Rust 크레이트를 썼다. 그런데 이걸 걷어내고 LibRaw C++ 소스를 직접 컴파일하기로 했다.

**왜?**

- `rsraw` 크레이트가 지원하지 않는 포맷이 있었을 것
- 버전 업데이트가 느려서 최신 카메라 지원이 안 되었을 것
- LibRaw를 직접 빌드하면 패치를 적용하거나 빌드 옵션을 제어할 수 있음
- `libraw_shim.cpp`를 직접 작성해서 필요한 API만 노출

**이것 때문에 cmake가 필요해진 것**이고, **네가 PR에 추가한 이유**가 바로 이 시점의 결정 때문이다.

---

## Phase 7: "Windows 호환성 전쟁" (2026.02 ~ 03)

커밋 히스토리에서 가장 고통스러워 보이는 부분:

```
f522d39 build(windows): switch ffmpeg to vcpkg and bundle ffmpeg tools
e796dd9 build(windows): use dynamic ffmpeg dlls bundled at app root
955117b ci(windows): copy all ffmpeg dll files from vcpkg bin
...
8a9ad98 Update workflow for Windows build 0.1.12
512635a Update workflow for windows again and again  ← 커밋 메시지에 고통이 묻어남
```

**"again and again"** — 이 커밋 메시지 하나에 Windows CI의 고통이 다 들어있다.

**무슨 일이 있었나?**

1. FFmpeg을 macOS처럼 정적 빌드하려 했지만, MSVC 툴체인 호환성 문제
2. vcpkg로 전환했지만 DLL 번들링 문제
3. 결국 동적 링킹 + DLL 직접 번들로 결정
4. CI 워크플로우를 수십 번 수정

이 과정을 거치면서 `Cargo.toml`에 플랫폼별 분기가 생겼다:

```toml
# Windows: 동적 링킹 (고통의 결과)
[target.'cfg(target_os = "windows")'.dependencies]
ffmpeg-next = { version = "8.1.0" }

# Unix: 정적 링킹 (이쪽은 잘 됨)
[target.'cfg(not(target_os = "windows"))'.dependencies]
ffmpeg-next = { features = ["static", "build"] }
```

---

## Phase 8: "Fullscreen 버그와의 전쟁" (2026.03 말)

```
8d22e1a fix: Fullscreen issue in Windows
94e886f fix: Fullscreen issue in Windows
80c3572 fix: FullScreen issue in Windows
```

같은 "Fullscreen issue in Windows"를 **3번 연속 수정**. 데스크탑 앱에서 Windows WebView2의 전체화면은 OS마다 동작이 달라서 악명 높은 문제다.

---

## 기술 선택의 패턴: 이 개발자의 의사결정 방식

커밋 히스토리에서 보이는 일관된 패턴:

### 1. "먼저 해보고, 안 되면 바꾼다"

```
xgplayer → video.js
Cropper.js → 자체 구현
rsraw → LibRaw 소스 빌드
직접 만든 trash → OS trash crate
base64 data URL → custom protocol
Tauri 1.0 → Tauri 2.0
```

라이브러리를 먼저 도입해보고, 맞지 않으면 과감하게 교체한다. sunk cost fallacy(매몰 비용)에 빠지지 않는다.

### 2. "유저가 뭘 설치해야 하는지 최소화한다"

- Python 필요 없음 (ONNX Runtime)
- DB 서버 필요 없음 (SQLite)
- 시스템 FFmpeg 필요 없음 (정적 링킹)
- 외부 라이브러리 필요 없음 (LibRaw 소스 빌드)

### 3. "데이터는 한 곳에"

- 메타데이터, 썸네일, AI 벡터, 얼굴 데이터 → 전부 SQLite
- 설정 → JSON 파일
- 별도 캐시 디렉토리 없음

### 4. "보안은 구조적으로"

- WebView에서 파일 경로 직접 노출하지 않음 (custom protocol)
- capability 기반 권한 (Tauri 2)
- 유저가 명시적으로 허용한 폴더만 접근

### 5. "성능 문제는 겪은 후에 해결한다"

- Virtual Scrolling은 중반부에 도입
- Custom protocol은 v0.1.12에서야 도입
- LRU 캐시는 RAW 지원 후에 추가
- 처음부터 최적화하지 않고, 문제가 드러나면 해결

---

## 왜 오픈소스로 공개했을까?

커밋 히스토리를 보면:

```
86a101a add license files (GPL-3.0)
28f0909 update README.md: add screenshot
ba25e39 update: remove album moved to edit album dialog
ad454f8 feat: init vitepress website and deploy workflow
```

GPL-3.0 라이선스, README 스크린샷, VitePress 문서 사이트까지 만든 건 **공개를 염두에 두고 준비한 것**이다. 2024년 8월에 시작해서 2026년 2월에 v0.1.0을 공개하기까지 **18개월간** 혼자 개발한 후 공개했다.

18개월 동안 혼자 614개 커밋을 쌓으면서:

- "bugfix" 커밋이 수백 개
- "again and again" 같은 고통의 커밋
- 도입했다 걷어낸 라이브러리들
- 3번 연속 같은 버그 수정

그런데도 포기하지 않고 꾸준히 기능을 추가하고, 최종적으로 깔끔한 아키텍처를 완성했다.

> **이 프로젝트가 대단한 이유는 아키텍처가 처음부터 완벽했기 때문이 아니다. 삽질하고, 되돌리고, 더 나은 방법을 찾아가는 과정을 614번 반복했기 때문이다.**

