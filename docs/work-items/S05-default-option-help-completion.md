# S05: 기본 UI 주요 옵션 도움말 보완

- 상태: implemented / compile and private GUI verification pending
- 분류: feature
- 우선순위: P0
- 요구사항: R07
- 담당 파일: soak_recovery_audit — crates/app/src/main.rs, crates/app/src/catalog.rs, 이 문서
- 의존성: S05-default-option-help-gap 및 기존 typed 옵션/scrollable Downloads

## 범위와 인수 조건

기본UI숫자품질0/144~4320의전용중앙help5요소와range/cut/FFmpeg기존metadata를field근처13pxtext로출력한다. 기존입력tab순서/scroll과CPUhelp정책은유지한다. 소유파일rustfmt만실행하고Cargo/GUI는root인수한다. Core/schema/다른파일/문구복제tests추가없음.

## 구현과 동결

FFmpeg·구간·자르기의 기존 중앙 default/example/conflict/unverified metadata를 기본 main.rs field 옆13px text로 연결했다. 숫자품질은전용DEFAULT_QUALITY3상수로0/144~4320/예1080/원본상한/변환없음/Audio미지원/미확인버전을명시했다. CPU의Audio품질문구를재사용하지않으며기존CPU상수/출력은수정하지않았다.

기존input/pick_list/Message순서와단일scrollable column은유지했다. 추가text는interactive focus를추가하지않지만페이지길이를늘리므로privateGUI scroll/keyboard/가시성은root후속인수다. 문구복제test나core/schema/dependency추가는없다.

소유2파일 `rustfmt --edition 2024` 실제exit0. SHA256 main.rs `f72ad934d523f0d3b2ea65e66ffd330022051e0e35239592eaac929b69b06587`, catalog.rs `30de2a5af510cf3031e8571f87b8ddebb1d51e26e448688287cc8d161c81f565`. Cargo/GUI를실행하지않았고compile/실제가시성은미검증이다. Sourcefreeze를root에보고해전체compile/기존옵션의미tests/격리GUI후속을맡겼다.
