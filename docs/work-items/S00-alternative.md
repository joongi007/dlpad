# S00: egui/eframe 대안 비교

- 상태: implemented, 통합 build와 GUI 측정 대기
- 분류: experiment
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: library agent — crates/app/src/egui_spike.rs, 이 문서
- 선행 작업: S00 Iced probe의 접근성 bridge 확인

## 사용자 결과

동일한 한국어 URL·IME 입력, System/Light/Dark, 100/150/200% 배율과 합성 10,000행 목록을 native OS 창에서 비교한다.

## 계약과 범위

PRODUCT, DESIGN, UX, STACK, PERFORMANCE, ADR-0001을 읽고 impeccable product register를 적용했다. DESIGN의 명시적 색상 기준은 skill 기본색보다 우선한다. 기존 중앙 catalog를 재사용한다. 독립 binary의 main이며 주 담당이 optional eframe 0.33.3 manifest/lock 통합을 소유한다. default_fonts/glow/wayland/x11/accesskit feature만 활성화하며 제품 backend 확정은 실측 뒤 결정한다. ScrollArea.show_rows는 viewport 행만 생성하며 전체 제목 배열을 만들지 않는다. continuous request_repaint를 사용하지 않는다. 네트워크 폰트 로딩이나 Documents 접근은 없다.

OS 한글 폰트의 알려진 경로와 DLPAD_TEST_FONT를 사용하고 파일 32 MiB 상한을 둔다. 이는 production OS font discovery가 아니며 glyph coverage를 확인하지 않았으므로 한글 지원 완료라고 표현하지 않는다. alternative 테마 유지와 파일 저장은 현재 제외이며 기존 probe와 동일한 영속 설정은 후속 통합 대상이다.

## 인수 조건

`cargo run -p dlpad --features alternative --bin dlpad-egui-spike -- --fixture`로 standalone native 창이 열리고 가상화 목록·입력·배율·테마를 비교한다. accesskit feature 존재는 스크린리더 성공과 구별한다. OS별 실제 한글 조합, 스크린리더, 배율, theme change와 메모리/CPU 결과를 주 담당이 기록한다.

## 검증과 handoff

공식 [NativeOptions 0.33.3 문서](https://docs.rs/eframe/0.33.3/eframe/struct.NativeOptions.html)와 [egui Context](https://docs.rs/egui/0.33.3/egui/struct.Context.html)를 확인했다. Cargo 동시 실행은 하지 않는다. GUI/IME/접근성/성능 미검증. glow는 GPU/OpenGL 드라이버 비용이 있으며 accesskit는 OS 접근성 adapter 의존성과 binary 비용을 더한다. release 크기/메모리 실측 전 비용 우열을 주장하지 않는다.
