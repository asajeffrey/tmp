use euclid::default::Size2D;
use sparkle::gl;
use sparkle::gl::Gl;
use std::rc::Rc;
use surfman::Context;
use surfman::Device;
use surfman::Surface;

fn challenge(gl: &Gl, device: &mut Device, context: &mut Context, surface: Surface) -> Surface {
    // THE GOAL IS TO BLIT THE PIXELS OUT OF SURFACE AND INTO CONTEXT
    // You can assume that there is a surface bound to the contezt,
    // of the same size as the surface.
    device
        .make_context_current(context)
        .expect("Failed to make current");

    let size = device.context_surface_info(&context).unwrap().unwrap().size;
    let surface_texture = device.create_surface_texture(context, surface).unwrap();
    let texture = surface_texture.gl_texture();

    let read_fbo = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::READ_FRAMEBUFFER, read_fbo);
    gl.framebuffer_texture_2d(
        gl::READ_FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        device.surface_gl_texture_target(),
        texture,
        0,
    );
    assert_eq!(
        (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );

    let draw_fbo = device
        .context_surface_info(&context)
        .unwrap()
        .unwrap()
        .framebuffer_object;
    gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, draw_fbo);
    assert_eq!(
        (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );

    gl.viewport(0, 0, size.width, size.height);
    gl.clear_color(0.2, 0.3, 0.3, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.blit_framebuffer(
        0,
        0,
        size.width,
        size.height,
        0,
        0,
        size.width,
        size.height,
        gl::COLOR_BUFFER_BIT,
        gl::NEAREST,
    );
    assert_eq!(
        (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );

    device
        .destroy_surface_texture(context, surface_texture)
        .unwrap()
}

const SIZE: Size2D<i32> = Size2D::new(2, 2);

fn init() -> (Rc<Gl>, u32, Device, Context) {
    let version = surfman::GLVersion { major: 4, minor: 3 };
    let flags = surfman::ContextAttributeFlags::ALPHA;
    let attributes = surfman::ContextAttributes { version, flags };

    let connection = surfman::Connection::new().expect("Failed to create connection");
    let adapter = surfman::Adapter::default().expect("Failed to create adapter");
    let mut device = surfman::Device::new(&connection, &adapter).expect("Failed to create device");
    let descriptor = device
        .create_context_descriptor(&attributes)
        .expect("Failed to create descriptor");
    let surface_type = surfman::SurfaceType::Generic { size: SIZE };

    let mut context = device
        .create_context(&descriptor)
        .expect("Failed to create context");
    let gl = Gl::gl_fns(gl::ffi_gl::Gl::load_with(|s| {
        device.get_proc_address(&context, s)
    }));
    device
        .make_context_current(&mut context)
        .expect("Failed to make current");
    assert_eq!(
        (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );
    let surface = device
        .create_surface(&mut context, surfman::SurfaceAccess::GPUCPU, &surface_type)
        .expect("Failed to create surface");
    device
        .bind_surface_to_context(&mut context, surface)
        .expect("Failed to bind surface");
    let fbo = device
        .context_surface_info(&context)
        .unwrap()
        .unwrap()
        .framebuffer_object;
    gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
    assert_eq!(
        (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );
    (gl, fbo, device, context)
}

fn main() {
    let (gl1, _, device1, mut context1) = init();
    let (gl2, fbo2, mut device2, mut context2) = init();
    
    device1
        .make_context_current(&mut context1)
        .expect("Failed to make current");
    gl1.clear_color(1.0, 0.0, 0.0, 1.0);
    gl1.clear(gl::COLOR_BUFFER_BIT);
    assert_eq!(
        (gl1.check_framebuffer_status(gl::FRAMEBUFFER), gl1.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );
    let surface1 = device1
        .unbind_surface_from_context(&mut context1)
        .expect("Failed to unbind surface")
        .expect("Failed to unbind surface");

    device2
        .make_context_current(&mut context2)
        .expect("Failed to make current");
    gl2.clear_color(0.5, 0.5, 0.5, 1.0);
    gl2.clear(gl::COLOR_BUFFER_BIT);
    assert_eq!(
        (gl2.check_framebuffer_status(gl::FRAMEBUFFER), gl2.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );

    // At this point surface1 is red, and surface2 is grey
    // The challege is to get surface2 to be red too!
    let surface1 = challenge(&gl2, &mut device2, &mut context2, surface1);

    gl2.bind_framebuffer(gl::FRAMEBUFFER, fbo2);
    assert_eq!(
        (gl2.check_framebuffer_status(gl::FRAMEBUFFER), gl2.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );
    let data = gl2.read_pixels(0, 0, SIZE.width, SIZE.height, gl::BGRA, gl::UNSIGNED_BYTE);
    assert_eq!(data, [0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255]);
    assert_eq!(
        (gl2.check_framebuffer_status(gl::FRAMEBUFFER), gl2.get_error()),
        (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
    );

    let surface2 = device2
        .unbind_surface_from_context(&mut context2)
        .expect("Failed to unbind surface")
        .expect("Failed to unbind surface");

    device1
        .destroy_surface(&mut context1, surface1)
        .expect("Failed to destroy surface");
    device2
        .destroy_surface(&mut context2, surface2)
        .expect("Failed to destroy surface");
    device1
        .destroy_context(&mut context1)
        .expect("Failed to destroy context");
    device2
        .destroy_context(&mut context2)
        .expect("Failed to destroy context");
}
