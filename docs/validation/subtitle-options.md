# Typed 자막 옵션 검증

2026-10-01 Linux/WSL, Rust 1.94.0, 기존 외부 yt-dlp 2026.09.27.232945를 사용했다. [공식 subtitle 옵션](https://github.com/yt-dlp/yt-dlp#subtitle-options)과 실제 설치 `--help`를 확인했다. 실제 사용자 영상·Documents·인증·외부 사이트는 사용하지 않았다.

## 구현 계약

DownloadOptions.subtitles는 optional SubtitleOptions다. SubtitleLanguage의 유한 catalog는 한국어(ko), 영어(en), 한국어와 영어(ko,en)이며 regex/all/live_chat/임의 문자열을 받지 않는다. exact language tag만 선택하므로 사이트가 en-orig 등 다른 tag만 제공하면 선택 언어가 없을 수 있다. extractor metadata 기반 원본 언어 선택은 미지원이다.

선택 시 `--write-subs --sub-langs <finite codes> --sub-format vtt/best`를 추가한다. 자동 생성 자막은 별도 true일 때만 `--write-auto-subs`를 추가한다. sidecar 파일이며 embedding/변환을 수행하지 않는다. 저장안함이 기본값이며 추가 요청·디스크 비용과 언어가 제공되지 않을 수 있음을 SUBTITLE_HELP에 설명한다. 설치 버전별 옵션 검증은 일반적으로 검증되지 않음이며 아래 합성 smoke 버전만 실증했다. 기존 numeric CATALOG는 유지하고 typed subtitle help를 분리했다.

자막 원본 시간축을 잘못 구간 영상에 맞는 것으로 표시하지 않도록 구간+자막 요청을 현재 거절한다. StoredOptions v1에는 optional serde(default,skipNone) 필드를 추가했으며 기존 무자막 JSON과 argv snapshot은 동일하다. unknown language/field/잘못된 bool을 거절하고 raw advanced subtitle override도 거절한다. dependency/schema 변경은 없다.

## 실행과 결과

- `rustfmt --edition 2024 crates/core/src/options.rs`: 성공.
- `cargo test -p dlpad-core --locked`: 20개 통과, 0개 실패, 본문 0.02초. 모든 유한 language/auto 조합, 도움말 catalog, argv 단일 flag, 저장 roundtrip, legacy 필드 생략, 손상 입력·reserved override·구간 충돌 검증 포함.
- `cargo clippy -p dlpad-core --all-targets --locked -- -D warnings`: 성공, 4.73초.
- `python3 scripts/subtitle_smoke.py <existing-yt-dlp-executable>` 실제 loopback smoke: Python 표준 라이브러리 임시 서버 127.0.0.1:OS선택port, synthetic .info.json과 두 VTT, 기존 yt-dlp argv 실행, 각 invocation 20초 timeout. auto=False는 synthetic.ko.vtt 한 개, auto=True는 synthetic.ko.vtt와 synthetic.en.vtt 두 개. 내용도 fixture bytes와 일치했고 0.82초에 통과했다.

실증 info는 `subtitles.ko`에 manual.vtt, `automatic_captions.en`에 automatic.vtt를 지정했다. engine 네트워크 extractor 대신 test-only `--skip-download --load-info-json <owned temporary metadata>`를 사용했다. argv에 typed flags, ignore-config/no-plugin-dirs/retries3, owned temporary --paths와 `--output synthetic.%(ext)s`를 적용했다. 모든 서버·자식·임시 폴더를 종료/정리했다. 이 시험은 실제 사이트 extractor나 영상 다운로드 성공을 증명하지 않는다.

## 최초 typed/helper 검증 시점의 후속

아래는 최초 검증 시점의 남은 작업이다. 이후 실제 연결 결과는 다음 단락과 최신 작업 계약을 따른다. 최초에는 앱 UI 입력/도움말과 최종 자막 artifact가 미연결이며 engine도 media marker 하나만 검증했다. 자막은 요청 메타데이터의 requested_subtitles filename/ext/language에서 최대 선택 언어 두 개로 제한하여 증거를 모으고, owned staging regular/nonempty/symlink 제외와 확장자 정책을 검증해야 한다. 미디어+sidecar publication plan을 먼저 journal에 저장하고 crash-gap recovery 및 artifact 등록 이후에만 Downloaded로 전이해야 한다. 구간 자막 시간축 재작성, 원본 언어 metadata 선택, 실제 사이트/다른 OS는 남은 게이트다. R07 전체 완료 상태가 아니다.

## 최신 production 연결

[다중 artifact 작업](../work-items/S05-artifact-batch.md)에서 영상+최대2개자막 publication journal v5, 전체 묶음 완료, Unix identity crash복구와 네이티브 자막 선택 UI를 연결했다. after_video projection은 id 및 한/영 filepath/ext만 받으며 작은 증거에signed 자막URL을 포함하지 않는다. 실제yt-dlp 합성 loopback projection과 real yt-dlp/FFmpeg 영상+VTT engine/DB reopen 시험이통과했다. 자막선택/자동생성/구간충돌/실행중freeze 가상GUI는별도담당이실증중이다. 구간자막시간축재작성, 원본언어metadata선택, 실제사이트/다른OS/공식관리pin버전별검증은남는다.
